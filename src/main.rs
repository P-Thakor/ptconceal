use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use walkdir::WalkDir;
use zip::{CompressionMethod, write::FileOptions};

#[derive(Parser)]
#[command(name = "ptconcealer")]
#[command(about="CLI app to conceal and reveal directories in and from videos", long_about = None)]
struct Cli {
    #[command(subcommand)]
    action: ConcealerAction,
}

#[derive(Subcommand)]
enum ConcealerAction {
    Conceal {
        directory: PathBuf,
        in_video: PathBuf,
    },
    Reveal {
        from_video: PathBuf,
    },
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    match args.action {
        ConcealerAction::Conceal {
            directory,
            in_video,
        } => {
            let zip_path = zip_directory(&directory)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
            println!("Successfully created: {}", zip_path.display());

            let result = conceal_in_video(&zip_path, &in_video);

            // Clean up temporary zip file regardless of success or failure
            if let Err(e) = std::fs::remove_file(&zip_path) {
                eprintln!("Warning: failed to clean up temporary zip file: {}", e);
            }

            result?;
            println!("Successfully appended zip file to the carrier video");
        }
        ConcealerAction::Reveal { from_video } => {
            let output_dir = from_video.parent().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Video path has no parent directory")
            })?;

            let zip_start_pos = extract_zip_from_video(&from_video, output_dir)?;
            println!("Extracted zip file successfully.");

            restore_video_file(&from_video, zip_start_pos)?;
            println!("Video file restored successfully.");
        }
    }
    Ok(())
}

fn zip_directory(src_dir: &Path) -> zip::result::ZipResult<PathBuf> {
    if !src_dir.is_dir() {
        return Err(zip::result::ZipError::FileNotFound);
    }

    let dest_file = src_dir.with_extension("zip");

    let file = File::create(&dest_file)?;
    let mut zip = zip::ZipWriter::new(file);

    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated) // use stored for no compression
        .unix_permissions(0o755);

    let mut buffer = [0u8; 8192];

    for entry in WalkDir::new(src_dir) {
        let entry = entry.map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to read directory entry: {}", e))
        })?;
        let entry_path = entry.path();
        let name = entry_path.strip_prefix(src_dir).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to strip path prefix: {}", e))
        })?;

        if entry_path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = BufReader::new(File::open(entry_path)?);
            loop {
                let bytes_read = f.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                zip.write_all(&buffer[..bytes_read])?;
            }
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }

    zip.finish()?;
    Ok(dest_file)
}

fn conceal_in_video(zip_path: &Path, video_path: &Path) -> io::Result<()> {
    let vid_file = OpenOptions::new().append(true).open(video_path)?;
    let mut writer = BufWriter::new(vid_file);

    let zip_file = File::open(zip_path)?;
    let mut reader = BufReader::new(zip_file);

    let mut buffer = [0u8; 8192]; // 8Kb chunk

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(&buffer[..bytes_read])?;
    }

    writer.flush()?;
    Ok(())
}

fn extract_zip_from_video(video_path: &Path, output_path: &Path) -> io::Result<u64> {
    // Zip End-of-Central-Directory signature
    const EOCD_SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];
    // EOCD fixed size (22 bytes) + max comment length (65535 bytes)
    const MAX_EOCD_SEARCH: u64 = 65557;

    let file = File::open(video_path)?;
    let file_len = file.metadata()?.len();

    if file_len < 22 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File too small to contain a zip archive",
        ));
    }

    // Read only the tail of the file where the EOCD must reside
    let search_len = file_len.min(MAX_EOCD_SEARCH);
    let search_start = file_len - search_len;

    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(search_start))?;

    let mut tail = vec![0u8; search_len as usize];
    reader.read_exact(&mut tail)?;

    // Scan backwards for the last EOCD signature
    let eocd_offset_in_tail = tail
        .windows(EOCD_SIGNATURE.len())
        .rposition(|w| w == EOCD_SIGNATURE)
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "No zip data found in video")
        })?;

    // Parse EOCD to find central directory size and offset
    let eocd = &tail[eocd_offset_in_tail..];
    if eocd.len() < 22 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Truncated End-of-Central-Directory record",
        ));
    }

    let cd_size = u32::from_le_bytes([eocd[12], eocd[13], eocd[14], eocd[15]]) as u64;
    let cd_offset = u32::from_le_bytes([eocd[16], eocd[17], eocd[18], eocd[19]]) as u64;

    // The central directory offset is relative to the archive start, so:
    // zip_start + cd_offset + cd_size == eocd_absolute_position
    let eocd_pos = search_start + eocd_offset_in_tail as u64;
    let zip_start_pos = eocd_pos
        .checked_sub(cd_offset + cd_size)
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "Invalid zip structure: offsets overflow")
        })?;

    // Extract the zip portion to a temporary file
    drop(reader);
    let output_file = output_path.join("concealed_data.zip");
    let mut file = File::open(video_path)?;
    file.seek(SeekFrom::Start(zip_start_pos))?;

    let mut writer = BufWriter::new(File::create(&output_file)?);
    io::copy(&mut file, &mut writer)?;
    writer.flush()?;

    // Validate and extract the archive
    let zip_file = File::open(&output_file)?;
    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Invalid zip: {}", e)))?;

    archive.extract(output_path)?;

    std::fs::remove_file(output_file)?;

    Ok(zip_start_pos)
}

fn restore_video_file(video_path: &Path, zip_start_pos: u64) -> io::Result<()> {
    let video_file = OpenOptions::new().write(true).open(video_path)?;
    video_file.set_len(zip_start_pos)?;
    Ok(())
}
