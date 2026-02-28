use std::path::Path;

use once_cell::sync::Lazy;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::errors::error::SubstrateError;

#[cfg(feature = "logging")]
use tracing::{error, trace};

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to create HTTP client")
});

/// Download a file from the given URL to the specified output path.
///
/// # Arguments
/// * `url` - The URL of the file to download.
/// * `outpath` - The path to save the downloaded file.
pub async fn download_helper(url: &str, outpath: &Path) -> Result<(), SubstrateError> {
    if url.is_empty() {
        #[cfg(feature = "logging")]
        error!("Url is empty");
        return Err(SubstrateError::NotFound {
            resource: "Url cant be empty".to_string(),
        });
    }

    #[cfg(feature = "logging")]
    trace!("Get HTTP client cache");
    let mut resp = HTTP_CLIENT.get(url).send().await?;

    if !resp.status().is_success() {
        #[cfg(feature = "logging")]
        error!("Response has not status success");
        return Err(SubstrateError::HttpStatus {
            body: resp.text().await?,
        });
    }

    #[cfg(feature = "logging")]
    trace!("Creating file");
    let file = tokio::fs::File::create(outpath).await?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);

    #[cfg(feature = "logging")]
    trace!("Writing file");
    while let Some(chunk) = resp.chunk().await? {
        writer.write_all(&chunk).await?;
    }

    writer.flush().await?;

    Ok(())
}
