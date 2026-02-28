use std::{collections::HashMap, path::PathBuf};

use crate::{
    check_java::check_java, download::download_helper::download_helper,
    download::download_java::download_java, errors::error::SubstrateError,
};

#[cfg(feature = "logging")]
use tracing::{error, info, trace, warn};

/// Config for the Minecraft server.
#[derive(serde::Deserialize)]
struct MinecraftConfig {
    versions: HashMap<String, HashMap<String, LoaderConfig>>,
}

/// Config for the Minecraft server.
#[derive(serde::Deserialize)]
struct LoaderConfig {
    url: String,
    java_version: String,
    loader: String,
}

/// Download a Minecraft server with the specified configuration.
///
/// # Arguments
/// * `name` - The name of the server.
/// * `loader` - The loader to use.
/// * `version` - The version of the server.
/// * `agree_eula` - Whether to agree to the EULA.
/// * `force_java_version` - The Java version to force.
/// * `current_dir` - The current project directory.
pub async fn download_server(
    name: &str,
    loader: &str,
    version: &str,
    agree_eula: bool,
    force_java_version: Option<String>,
    current_dir: PathBuf,
) -> Result<(String, String), SubstrateError> {
    if !agree_eula {
        #[cfg(feature = "logging")]
        {
            warn!("Skipping creation process due to eula is not value true");
            warn!("If you want to create a server you need to agree to the eula");
        }
        return Err(SubstrateError::Eula);
    }

    #[cfg(feature = "logging")]
    trace!("Joining servers dir");
    let servers_directory = &current_dir.join("servers");

    if !tokio::fs::try_exists(&servers_directory).await? {
        #[cfg(feature = "logging")]
        error!("servers directory does not exist");
        return Err(SubstrateError::AlreadyExists {
            resource: "Servers directory was not found in current_dir".to_string(),
        });
    }

    #[cfg(feature = "logging")]
    trace!("Joining server dir");
    let server_dir = &servers_directory.join(name);

    let config = {
        #[cfg(feature = "logging")]
        trace!("Loading minecraft.json configuration");
        let config_dir = &current_dir.join("minecraft.json");

        let config_str = tokio::fs::read_to_string(&config_dir).await?;

        let mut config: MinecraftConfig = serde_json::from_str(&config_str)?;

        #[cfg(feature = "logging")]
        trace!("Searching minecraft version");
        let mut inner = match config.versions.remove(version) {
            Some(v) => {
                #[cfg(feature = "logging")]
                trace!("Found right minecraft version");
                v
            }
            None => {
                #[cfg(feature = "logging")]
                error!("Failed to find version");
                return Err(SubstrateError::NotFound {
                    resource: "Could not find minecraft version in minecraft.json".to_string(),
                });
            }
        };

        #[cfg(feature = "logging")]
        trace!("Searching for loader");
        match inner.remove(loader) {
            Some(v) => {
                #[cfg(feature = "logging")]
                trace!("Found minecraft loader");
                v
            }
            None => {
                #[cfg(feature = "logging")]
                error!("Failed to find loader");
                return Err(SubstrateError::NotFound {
                    resource: "Could not find minecraft loader in minecraft.json".to_string(),
                });
            }
        }
    };

    #[cfg(feature = "logging")]
    trace!("Getting java version");
    let java_version = force_java_version.unwrap_or(config.java_version);

    if !check_java(&java_version, &current_dir).await? {
        #[cfg(feature = "logging")]
        info!("Java version not found installing...");
        download_java(&java_version, &current_dir).await?;
    }

    let server_jar = server_dir.join("server.jar");

    #[cfg(feature = "logging")]
    info!("Downloading server.jar");
    download_helper(&config.url, &server_jar).await?;

    #[cfg(feature = "logging")]
    trace!("Dropping server jar path");
    drop(server_jar);

    Ok((name.to_string(), java_version))
}
