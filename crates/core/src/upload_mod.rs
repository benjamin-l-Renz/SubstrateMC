use std::path::PathBuf;

use bytes::Bytes;

use tokio::io::AsyncWriteExt;
#[cfg(feature = "logging")]
use tracing::warn;
/// Represents a mod file containing the mod data and the output path.
pub struct File {
    bytes: Bytes,
    path: PathBuf,
}

use crate::errors::error::SubstrateError;

/// Uploads a mod to the file system of the server.
///
/// The given file is uploaded to the file system of the server into the specified path.
///
/// # Arguments
///
/// * `file` - The mod file containing the mod data and the output path.
pub async fn upload_mod(file: File) -> Result<(), SubstrateError> {
    if file.bytes.is_empty() {
        #[cfg(feature = "logging")]
        warn!("Mod file is empty");

        return Err(SubstrateError::UploadModError {
            message: "Mod file is empty".to_string(),
        });
    }

    let mut disk_file = tokio::fs::File::create(file.path).await?;

    disk_file.write_all(&file.bytes).await?;

    disk_file.flush().await?;

    Ok(())
}
