use crate::download::download_helper::download_helper;
use crate::rename::rename_unpacked_java_folder;
use crate::unpack::unpack;
use std::path::Path;

use crate::errors::error::SubstrateError;
#[cfg(feature = "logging")]
use tracing::{error, info, trace, warn};

#[derive(serde::Deserialize)]
struct JavaDownloadConfig {
    version: String,
    url: String,
}

#[derive(serde::Deserialize, Default)]
struct JavaConfig {
    linux: Vec<JavaDownloadConfig>,
}

/// The `download_java_locally` downloades a given version of the java runtime
///
/// # Arguments
/// * `version` - Java version
/// * `current_dir` - Current project root directory
pub async fn download_java(version: &str, current_dir: &Path) -> Result<(), SubstrateError> {
    #[cfg(feature = "logging")]
    trace!("Joining runtime dir");
    let runtime_dir = current_dir.join("runtime");

    if !tokio::fs::try_exists(&runtime_dir).await? {
        #[cfg(feature = "logging")]
        info!("Creating Runtime dir");
        tokio::fs::create_dir_all(&runtime_dir).await?;
    }

    #[cfg(feature = "logging")]
    trace!("Joining java.json config file");
    let config_file = current_dir.join("java.json");

    if !tokio::fs::try_exists(&config_file).await? {
        #[cfg(feature = "logging")]
        error!("Config file not found");
        return Err(SubstrateError::NotFound {
            resource: "Could not find config file java.json".to_string(),
        });
    }

    #[cfg(feature = "logging")]
    trace!("Reading java.json.");
    let bytes = tokio::fs::read(&config_file).await?;

    let config: JavaConfig = serde_json::from_slice(&bytes)?;

    #[cfg(feature = "logging")]
    trace!("Droping bytes");
    drop(bytes);

    #[cfg(feature = "logging")]
    trace!("Searching in config...");
    let config =
        config
            .linux
            .iter()
            .find(|j| j.version == version)
            .ok_or(SubstrateError::NotFound {
                resource: "could not find requested java version".to_string(),
            })?;

    // Get the archive path and the target path
    let archive_path = &runtime_dir.join(format!("java-{}.tar.gz", &config.version));
    let java_name = format!("java-{}", config.version);

    // if java is downloaded async you would need to check if there is already an archive
    if tokio::fs::try_exists(&archive_path).await? {
        #[cfg(feature = "logging")]
        warn!("Java version {} already downloaded", config.version);
        return Ok(());
    }

    // Check if java version is already downloaded
    if tokio::fs::try_exists(&runtime_dir.join(&java_name)).await? {
        #[cfg(feature = "logging")]
        warn!("Java version {} already downloaded", config.version);
        return Ok(());
    }

    #[cfg(feature = "logging")]
    info!("Downloading Java");
    download_helper(&config.url, archive_path).await?;

    #[cfg(feature = "logging")]
    trace!("Unpacking Java");
    unpack(archive_path, &runtime_dir).await?;

    #[cfg(feature = "logging")]
    trace!("Dropping archive path");

    #[cfg(feature = "logging")]
    trace!("Creating target path");
    let target_path = runtime_dir.join(&java_name);

    #[cfg(feature = "logging")]
    trace!("Renaming unpacked folder...");
    rename_unpacked_java_folder(&target_path, &java_name, &runtime_dir).await?;

    // TODO: delete archive path

    Ok(())
}
