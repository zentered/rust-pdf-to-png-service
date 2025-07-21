# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

### Building the Project
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Install binary to cargo bin directory
cargo install --path .
```

### Running the Service
```bash
# For local development
RUSTFLAGS="-C target-feature=-crt-static $(pkg-config vips --libs)" cargo build && ./target/debug/pdf-service

# Set required environment variables
export INPUT_PATH=/path/to/input/pdfs
export OUTPUT_PATH=/path/to/output/images
export PORT=8080
```

### Docker Development
```bash
# Build Docker image
docker build -t pdf-service .

# Run container with environment variables and volume mounts
docker run -e INPUT_PATH=/input -e OUTPUT_PATH=/output -e PORT=8080 \
  -v /host/input:/input -v /host/output:/output pdf-service
```

## Architecture Overview

This is a Rust-based PDF to PNG/WebP conversion service designed for local file system operations. The service processes PDF files from a local input directory and generates multiple image formats/sizes to an output directory using atomic file operations.

### Core Components

**Main Service (`src/main.rs:41-150`)**
- Axum HTTP server that receives HTTP triggers
- Processes POST requests with Cloud Event headers
- Reads PDF from input directory, converts to images, writes to output directory atomically
- Generates 6 variants: original PNG, full-size PNG, preview PNG, thumbnail PNG, WebP, and lossless WebP

**PDF Conversion (`src/convert.rs`)**
- Uses `pdfium-render` to convert PDF first page to DynamicImage
- Configured for 3000px max width/height with landscape rotation
- Requires libpdfium.dylib/libpdfium.so in `./lib/` directory

**Image Processing (`src/transform.rs`)**
- Uses libvips for high-performance image transformations
- Format variants:
  - Original: 90% quality PNG
  - Preview: 1920px width, 75% quality, interlaced PNG  
  - Thumbnail: 1024px width, 70% quality, interlaced PNG
  - WebP: 1920px width, 75% quality
  - WebP Lossless: 100% quality

**File Operations (`src/file_ops.rs`)**
- Reads PDFs from INPUT_PATH using local file system
- Writes processed images to OUTPUT_PATH using atomic operations
- Uses temporary directories to ensure atomic writes
- Custom error handling for file operations

### Key Dependencies

- **pdfium-render**: PDF rendering (requires libpdfium binary)
- **libvips**: High-performance image processing
- **uuid**: Unique temporary directory naming
- **axum**: HTTP server framework
- **image**: Image manipulation utilities

### Environment Requirements

- `INPUT_PATH`: Local directory path for input PDFs
- `OUTPUT_PATH`: Local directory path for output images  
- `PORT`: HTTP server port

### File Processing Flow

1. Reads PDF from `INPUT_PATH/{filename}`
2. Creates unique temporary directory under system temp
3. Generates all image variants in temporary directory
4. Atomically moves all files to `OUTPUT_PATH`
5. Cleans up temporary directory

The service expects libpdfium shared libraries in `./lib/` and requires libvips system libraries. The Dockerfile handles all system dependencies automatically.