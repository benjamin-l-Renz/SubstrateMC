#[cfg(feature = "logging")]
use tracing::error;

#[cfg(feature = "logging")]
use tracing::info;

use crate::{download::download, errors::error::ApiError, unpack::unpack};

#[derive(serde::Deserialize, Default)]
pub struct JavaConfig {
    pub linux: Vec<JavaDownloadConfig>,
}

#[derive(serde::Deserialize)]
pub struct JavaDownloadConfig {
    pub version: String,
    pub url: String,
}

pub async fn download_java_locally(version: &str) -> Result<(), ApiError> {
    let project_dir = std::env::current_dir()?;
    let config_dir = project_dir.join("java.json");
    let runtime_dir = project_dir.join("runtime");

    if !tokio::fs::try_exists(&runtime_dir).await? {
        #[cfg(feature = "logging")]
        info!("Create runtime dir");
        tokio::fs::create_dir_all(&runtime_dir).await?;
    }

    if !tokio::fs::try_exists(&config_dir).await? {
        #[cfg(feature = "logging")]
        error!("Java config file not found at {:?}", config_dir);
        return Err(ApiError::NotFound);
    }

    #[cfg(feature = "logging")]
    info!("reading java config file");
    let config_data = tokio::fs::read_to_string(config_dir).await?;

    let java_config: JavaConfig = serde_json::from_str(&config_data)?;

    let java = java_config
        .linux
        .iter()
        .find(|j| j.version == version)
        .ok_or(ApiError::NotFound)?;

    // Archive path where the downloaded file will be stored
    let archive_path = runtime_dir.join(format!("java-{}.tar.gz", java.version));

    let java_path = runtime_dir.join(format!("java-{}", java.version));

    if tokio::fs::try_exists(&java_path).await? {
        #[cfg(feature = "logging")]
        info!("Java version {} already downloaded", java.version);
        return Ok(());
    }

    let archive_path_str = archive_path.to_str().ok_or(ApiError::InternalServerError)?;
    let runtime_dir_str = runtime_dir.to_str().ok_or(ApiError::InternalServerError)?;

    #[cfg(feature = "logging")]
    info!(
        "downloading java version {} from {}",
        java.version, java.url
    );
    download(&java.url, archive_path_str).await?;

    #[cfg(feature = "logging")]
    info!("unpacking java version {}", java.version);
    unpack(archive_path_str, runtime_dir_str).await?;

    // Rename the unpacked directory to java-{version}
    let target_name = format!("java-{}", java.version);
    let target_path = runtime_dir.join(&target_name);

    rename_unpacked_java_folder(&target_path, target_name, &runtime_dir).await?;

    tokio::fs::remove_file(archive_path).await?;

    Ok(())
}

async fn rename_unpacked_java_folder(
    target_path: &std::path::PathBuf,
    target_name: String,
    runtime_dir: &std::path::PathBuf,
) -> Result<(), ApiError> {
    if !tokio::fs::try_exists(target_path).await? {
        let mut entries = tokio::fs::read_dir(runtime_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            #[cfg(feature = "logging")]
            info!("renaming archive");

            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && path.is_dir()
                && name != target_name
                && (name.to_uppercase().starts_with("JDK") || name.to_uppercase().contains("JDK"))
            {
                #[cfg(feature = "logging")]
                info!("Renaming {} to {}", name, target_name);
                tokio::fs::rename(path, target_path).await?;
                break; // Exit early once we find and rename the directory
            }
        }
    }

    Ok(())
}
