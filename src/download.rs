use once_cell::sync::Lazy;
use tokio::io::{AsyncWriteExt, BufWriter};
#[cfg(feature = "logging")]
use tracing::error;

use crate::errors::error::ApiError;

// Reusable HTTP client with connection pooling
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    match reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .build()
    {
        Ok(builder) => builder,
        Err(err) => {
            #[cfg(feature = "logging")]
            error!("Failed to create HTTP client: {}", err);
            panic!("Failed to create HTTP client: {}", err)
        }
    }
});

pub async fn download(url: &str, outpath: &str) -> Result<(), ApiError> {
    if url.is_empty() {
        return Err(ApiError::NotFound);
    }

    let mut resp = HTTP_CLIENT.get(url).send().await?;

    if !resp.status().is_success() {
        return Err(ApiError::InternalServerError);
    }

    let file = tokio::fs::File::create(outpath).await?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file); // 64KB buffer

    while let Some(chunk) = resp.chunk().await? {
        writer.write_all(&chunk).await?;
    }

    writer.flush().await?;

    Ok(())
}
