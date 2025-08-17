# ptconceal

ptconceal is a Rust-based project designed for file concealment and protection. It provides tools to compress and conceal files and directories in a carrier video file.

## Features
- File compression and extraction
- Secure file management
- Fast and efficient performance using Rust

## Getting Started

### Prerequisites
- Rust (https://www.rust-lang.org/tools/install)

### Build
To build the project, run:
```
cargo build --release
```
The executable will be located in the `target/release/` directory.

### Usage
Run the program using:
```
target\release\ptconceal.exe [options]
```
Refer to the command-line help for available options:
```
target\release\ptconceal.exe --help
```


## Releases

You can download pre-built binaries from the Releases section of the project's GitHub page.

### Installation

**Windows:**
1. Download the latest release binary (`ptconceal.exe`) from GitHub.
2. Place `ptconceal.exe` in `C:\Windows\System32`.
3. You can now run `ptconceal` from any terminal window as a CLI tool.

**Linux:**
1. Download the latest release binary from GitHub.
2. Place the binary in `/usr/local/bin` and ensure it is executable (`chmod +x /usr/local/bin/ptconceal`).
3. You can now run `ptconceal` from any terminal window as a CLI tool.

### Example
```
ptconceal [options]
```
Refer to the command-line help for available options:
```
ptconceal --help
```

## Author
P-Thakor
