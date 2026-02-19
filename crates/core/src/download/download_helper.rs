use once_cell::sync::Lazy;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::errors::error::SubstrateError;

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
pub async fn download_helper(url: &str, outpath: &str) -> Result<(), SubstrateError> {
    if url.is_empty() {
        return Err(SubstrateError::NotFound {
            resource: "Url cant be empty".to_string(),
        });
    }

    let mut resp = HTTP_CLIENT.get(url).send().await?;

    if !resp.status().is_success() {
        return Err(SubstrateError::HttpStatus {
            body: resp.text().await?,
        });
    }

    let file = tokio::fs::File::create(outpath).await?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);

    while let Some(chunk) = resp.chunk().await? {
        writer.write_all(&chunk).await?;
    }

    writer.flush().await?;

    Ok(())
}
