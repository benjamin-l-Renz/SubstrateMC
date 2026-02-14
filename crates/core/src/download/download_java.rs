use crate::download::download_helper::download_helper;
use crate::rename::rename_unpacked_java_folder;
use crate::unpack::unpack;
use std::path::PathBuf;

use crate::errors::api_error::ApiError;
#[cfg(feature = "logging")]
use tracing::info;

#[derive(serde::Deserialize)]
pub struct JavaDownloadConfig {
    pub version: String,
    pub url: String,
}

#[derive(serde::Deserialize, Default)]
pub struct JavaConfig {
    pub linux: Vec<JavaDownloadConfig>,
}

/// The `download_java_locally` function takes a version as a string slice and trys to install Java.
/// It expects a config file named java.json where the java versions are noted
pub async fn download_java(version: &str, current_dir: PathBuf) -> Result<(), ApiError> {
    let runtime_dir = current_dir.join("runtime");
    let config_file = current_dir.join("java.json");

    if !tokio::fs::try_exists(&runtime_dir).await? {
        #[cfg(feature = "logging")]
        info!("Creating Runtime dir");
        tokio::fs::create_dir_all(&runtime_dir).await?;
    }

    if !tokio::fs::try_exists(&config_file).await? {
        return Err(ApiError::NotFound("Couldnt find java.json".to_string()));
    }

    let config_str = tokio::fs::read_to_string(&config_file).await?;

    let config: JavaConfig = serde_json::from_str(&config_str)?;

    drop(config_str);

    let config = config
        .linux
        .iter()
        .find(|j| j.version == version)
        .ok_or(ApiError::NotFound(
            "Could not find requested java version in java.json".to_string(),
        ))?;

    // Get the archive path and the target path
    let archive_path = runtime_dir.join(format!("java-{}.tar.gz", config.version));
    let java_path = runtime_dir.join(format!("java-{}", config.version));

    // if java is downloaded async you would need to check if there is already an archive
    if tokio::fs::try_exists(&archive_path).await? {
        #[cfg(feature = "logging")]
        info!("Java version {} already downloaded", config.version);
        return Ok(());
    }

    // Check if java version is already downloaded
    if tokio::fs::try_exists(&java_path).await? {
        #[cfg(feature = "logging")]
        info!("Java version {} already downloaded", config.version);
        return Ok(());
    }

    download_helper(
        &config.url,
        archive_path.to_str().ok_or(ApiError::InternalServerError)?,
    )
    .await?;

    unpack(
        archive_path.to_str().ok_or(ApiError::InternalServerError)?,
        runtime_dir.to_str().ok_or(ApiError::InternalServerError)?,
    )
    .await?;

    let target_name = format!("java-{}", config.version);
    let target_path = runtime_dir.join(&target_name);

    rename_unpacked_java_folder(&target_path, &target_name, &runtime_dir).await?;

    Ok(())
}
