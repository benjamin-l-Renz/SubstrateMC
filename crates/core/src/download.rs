use once_cell::sync::Lazy;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::errors::api_error::ApiError;

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to create HTTP client")
});

pub async fn download(url: &str, outpath: &str) -> Result<(), ApiError> {
    if url.is_empty() {
        return Err(ApiError::NotFound("Url cant be empty".to_string()));
    }

    let mut resp = HTTP_CLIENT.get(url).send().await?;

    if !resp.status().is_success() {
        return Err(ApiError::InternalServerError);
    }

    let file = tokio::fs::File::create(outpath).await?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);

    while let Some(chunk) = resp.chunk().await? {
        writer.write_all(&chunk).await?;
    }

    writer.flush().await?;

    Ok(())
}
