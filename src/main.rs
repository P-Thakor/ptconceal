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
    let source_dir;
    let video_path;
    let output_dir;

    let args = Cli::parse();

    let concealer_result = match &args.action {
        ConcealerAction::Conceal {
            directory,
            in_video,
        } => {
            source_dir = directory;
            video_path = in_video;
            let zip_path = match zip_directory(&source_dir) {
                Ok(path) => {
                    println!("Successfully created: {}", path.display());
                    path
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    PathBuf::new()
                }
            };

            match conceal_in_video(zip_path.as_path(), video_path.as_path()) {
                Ok(()) => println!("Successfully appended zip file to the carrier video"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        ConcealerAction::Reveal { from_video } => {
            video_path = from_video;
            if let Some(parent) = video_path.parent() {
                output_dir = parent.to_path_buf();
                match extract_zip_from_video(video_path.as_path(), output_dir.as_path()) {
                    Ok(zip_start_pos) => {
                        println!("Extracted zip file successfully.");
                        match restore_video_file(video_path.as_path(), zip_start_pos) {
                            Ok(()) => println!("Video file restored successfully."),
                            Err(e) => eprintln!("Error restoring video file: {}", e),
                        };
                    }
                    Err(e) => {
                        eprintln!("Error encountered while extracting zip, {}", e);
                    }
                }
            }
        }
    };
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

    let mut buffer = Vec::new();

    for entry in WalkDir::new(src_dir) {
        let entry = entry.unwrap();
        let entry_path = entry.path();
        let name = entry_path.strip_prefix(src_dir).unwrap();

        if entry_path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(entry_path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
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
    const ZIP_MAGIC_NUMBER: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

    let mut reader = BufReader::new(File::open(video_path)?);
    let mut buffer = [0u8; 8192];
    let mut overlap = Vec::new();
    let mut pos_in_file = 0u64;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let search_area = if overlap.is_empty() {
            &buffer[..bytes_read]
        } else {
            overlap.extend_from_slice(&buffer[..bytes_read]);
            overlap.as_slice()
        };

        if let Some(found_pos) = search_area
            .windows(ZIP_MAGIC_NUMBER.len())
            .position(|window| window == ZIP_MAGIC_NUMBER)
        {
            let zip_start_pos = pos_in_file + found_pos as u64 - overlap.len() as u64;

            drop(reader);

            let output_file = output_path.join("concealed_data.zip");
            let mut file = File::open(video_path)?;
            file.seek(SeekFrom::Start(zip_start_pos))?;

            let mut writer = BufWriter::new(File::create(&output_file)?);
            io::copy(&mut file, &mut writer)?;
            writer.flush()?;

            let zip_file = File::open(&output_file)?;
            let mut archive = zip::ZipArchive::new(zip_file)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Invalid zip: {}", e)))?;

            archive.extract(output_path)?;

            std::fs::remove_file(output_file)?;

            return Ok(zip_start_pos);
        }

        if bytes_read >= ZIP_MAGIC_NUMBER.len() {
            overlap = buffer[bytes_read - (ZIP_MAGIC_NUMBER.len() - 1)..bytes_read].to_vec();
        } else {
            overlap = buffer[..bytes_read].to_vec();
        }

        pos_in_file += bytes_read as u64;
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "No Zip data found in video.",
    ))
}

fn restore_video_file(video_path: &Path, zip_start_pos: u64) -> io::Result<()> {
    let video_file = OpenOptions::new().write(true).open(video_path)?;
    video_file.set_len(zip_start_pos)?;
    Ok(())
}
