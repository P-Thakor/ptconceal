# ptconceal

A command-line tool that **hides files and folders inside a video file** and extracts them back out — all without breaking the video.

## What does it do?

`ptconceal` has two operations:

| Command | What it does |
|---|---|
| `conceal` | Compresses a folder and hides it in a video file. The video still plays normally, but your files are hidden inside it. |
| `reveal` | Finds the hidden data inside a video, extracts the original files, and restores the video back to its original state. |

---

## Getting Started

### Prerequisites

You need **Rust** installed on your machine if you want to build from scratch. If you don't, simply download my pre-compiled binary
from [Releases](https://github.com/P-Thakor/ptconceal/releases) page. For installing Rust:

1. Go to [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
2. Follow the instructions for your OS (it's a single command on Linux/macOS)
3. Verify with:
   ```
   rustc --version
   ```

### Build from source

Clone the repo and build:

```bash
git clone https://github.com/P-Thakor/ptconceal.git
cd ptconceal
cargo build --release
```

The compiled binary will be at:
- **Linux / macOS:** `target/release/ptconceal`
- **Windows:** `target\release\ptconceal.exe`

### Install the binary

Installing makes `ptconceal` available as a command from any directory.

**Linux / macOS:**
```bash
sudo cp target/release/ptconceal /usr/local/bin/
```

**Windows:**

Copy `target\release\ptconceal.exe` to a folder that's in your system `PATH` (e.g. `C:\Windows\System32`).

> Alternatively, you can download a pre-built binary from the [Releases](https://github.com/P-Thakor/ptconceal/releases) page and skip the build step entirely.

---

## Usage

### Conceal — Hide a folder inside a video

```
ptconceal conceal <DIRECTORY> <VIDEO_FILE>
```

| Argument | Description |
|---|---|
| `<DIRECTORY>` | Path to the folder you want to hide |
| `<VIDEO_FILE>` | Path to the carrier video file (`.mp4`, `.mkv`, `.avi`, etc.) |

**Example:**

```bash
ptconceal conceal ./secret-notes ./lecture.mp4
```

This will:
1. Compress `secret-notes/` into a zip archive
2. Hide the zip data inside `lecture.mp4`
3. Clean up the temporary zip file

After this, `lecture.mp4` still plays normally in any video player — but your folder is hidden inside it.

> [!IMPORTANT]
> This **modifies the video file in-place**. Make a backup of your video first if you want to keep the original untouched.

---

### Reveal — Extract hidden files from a video

```
ptconceal reveal <VIDEO_FILE>
```

| Argument | Description |
|---|---|
| `<VIDEO_FILE>` | Path to the video that contains hidden data |

**Example:**

```bash
ptconceal reveal ./lecture.mp4
```

This will:
1. Scan the video for hidden zip data
2. Extract the concealed files into the **same directory** as the video
3. Restore the video to its original size (removes the hidden data)

After this, you'll find your extracted folder alongside the video, and the video is back to its clean original state.

---

## Full Walkthrough

Here's a complete example from start to finish:

```bash
# 1. You have a folder with files you want to hide
ls ./my-project/
#   notes.txt  diagram.png  code.py

# 2. You have a video to use as the carrier
ls -lh ./cat-video.mp4
#   48M  cat-video.mp4

# 3. Conceal the folder inside the video
ptconceal conceal ./my-project ./cat-video.mp4
#   Successfully created: my-project.zip
#   Successfully hid zip file to the carrier video

# 4. The video is now slightly larger, but still plays fine
ls -lh ./cat-video.mp4
#   51M  cat-video.mp4        ← a bit bigger now

# 5. Share the video, put it on a USB drive, etc.
#    Nobody will know there are files hidden inside.

# 6. Later, extract the hidden files
ptconceal reveal ./cat-video.mp4
#   Extracted zip file successfully.
#   Video file restored successfully.

# 7. Your files are back, and the video is restored
ls ./my-project/
#   notes.txt  diagram.png  code.py
ls -lh ./cat-video.mp4
#   48M  cat-video.mp4        ← back to original size
```

---

## Command Reference

```
ptconceal --help
```

```
CLI app to conceal and reveal directories in and from videos

Usage: ptconceal <COMMAND>

Commands:
  conceal  Hide a directory inside a video file
  reveal   Extract hidden data from a video file
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

---

## Troubleshooting

| Problem | Solution |
|---|---|
| `No zip data found in video` | The video doesn't contain any hidden data, or the data was corrupted. |
| `File too small to contain a zip archive` | The file you pointed to is too small (< 22 bytes) to hold any zip content. |
| `No such file or directory` | Double-check that the paths you provided are correct. |
| `Permission denied` | On Linux/macOS, make sure you have read/write access to both the directory and the video file. Try with `sudo` if needed. |
| Video won't play after concealing | The video format should be fine — try a different player (e.g. VLC). Some players may complain about unexpected trailing data but will still play the video. |

---

## Supported Formats

- **Video:** Any format works as the carrier (`.mp4`, `.mkv`, `.avi`, `.mov`, `.webm`, etc.).
- **Directories:** Any folder with any file types inside it can be concealed.

---

## Author

**P-Thakor**

## License

This project is licensed under a Custom License — see the [LICENSE](LICENSE) file for details.
