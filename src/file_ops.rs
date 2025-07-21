use std::env;
use std::path::{Path, PathBuf};
use std::io;
use tokio::fs;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug)]
pub enum FileError {
    Io(io::Error),
    NotFound(String),
    InvalidPath(String),
}

impl From<io::Error> for FileError {
    fn from(error: io::Error) -> Self {
        FileError::Io(error)
    }
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::Io(err) => write!(f, "IO error: {}", err),
            FileError::NotFound(path) => write!(f, "File not found: {}", path),
            FileError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
        }
    }
}

impl std::error::Error for FileError {}

pub async fn read_pdf(filename: &str) -> Result<Vec<u8>, FileError> {
    debug!("reading file {:?}", filename);
    
    let input_path = env::var("INPUT_PATH")
        .map_err(|_| FileError::InvalidPath("INPUT_PATH environment variable not set".to_string()))?;
    
    let file_path = Path::new(&input_path).join(filename);
    
    if !file_path.exists() {
        return Err(FileError::NotFound(file_path.to_string_lossy().to_string()));
    }
    
    let pdf_buffer = fs::read(&file_path).await?;
    info!("Successfully read PDF file: {:?} ({} bytes)", file_path, pdf_buffer.len());
    
    Ok(pdf_buffer)
}

pub fn create_temp_dir() -> Result<PathBuf, FileError> {
    let temp_base = env::temp_dir();
    let unique_id = Uuid::new_v4();
    let temp_dir = temp_base.join(format!("pdf-service-{}", unique_id));
    
    std::fs::create_dir_all(&temp_dir)?;
    debug!("Created temporary directory: {:?}", temp_dir);
    
    Ok(temp_dir)
}

pub async fn write_images_atomic(temp_dir: &Path, files: Vec<(String, Vec<u8>)>) -> Result<(), FileError> {
    debug!("writing {} images to temp directory {:?}", files.len(), temp_dir);
    
    let output_path = env::var("OUTPUT_PATH")
        .map_err(|_| FileError::InvalidPath("OUTPUT_PATH environment variable not set".to_string()))?;
    
    let output_dir = Path::new(&output_path);
    
    // Ensure output directory exists
    fs::create_dir_all(output_dir).await?;
    
    // Write all files to temp directory first
    for (filename, content) in &files {
        let temp_file_path = temp_dir.join(filename);
        fs::write(&temp_file_path, content).await?;
        debug!("Wrote temp file: {:?}", temp_file_path);
    }
    
    // Atomic move all files from temp to output directory
    for (filename, _) in &files {
        let temp_file_path = temp_dir.join(filename);
        let output_file_path = output_dir.join(filename);
        
        // Try atomic rename first (same filesystem)
        if let Err(_) = fs::rename(&temp_file_path, &output_file_path).await {
            // Fallback to copy + remove for cross-filesystem moves
            fs::copy(&temp_file_path, &output_file_path).await?;
            fs::remove_file(&temp_file_path).await?;
        }
        
        info!("Successfully moved file to: {:?}", output_file_path);
    }
    
    Ok(())
}

pub async fn cleanup_temp_dir(temp_dir: &Path) -> Result<(), FileError> {
    debug!("cleaning up temporary directory: {:?}", temp_dir);
    
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir).await?;
        debug!("Removed temporary directory: {:?}", temp_dir);
    }
    
    Ok(())
}

fn get_content_type(filename: &str) -> &'static str {
    match filename.rsplit_once('.').map(|(_, ext)| ext) {
        Some("webp") => "image/webp",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        _ => "application/octet-stream",
    }
}