use std::path::PathBuf;

use bytes::Bytes;

use tokio::io::AsyncWriteExt;
#[cfg(feature = "logging")]
use tracing::warn;

pub struct File {
    bytes: Bytes,
    path: PathBuf,
}

use crate::errors::api_error::ApiError;

pub async fn upload_mod(file: File) -> Result<(), ApiError> {
    if file.bytes.is_empty() {
        #[cfg(feature = "logging")]
        warn!("Mod file is empty");

        return Err(ApiError::InternalServerError);
    }

    let mut disk_file = tokio::fs::File::create(file.path).await?;

    disk_file.write_all(&file.bytes).await?;

    disk_file.flush().await?;

    Ok(())
}
