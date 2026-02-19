use std::{collections::HashMap, path::PathBuf};

use crate::{
    check_java::check_java, download::download_helper::download_helper,
    download::download_java::download_java, errors::error::SubstrateError,
};

#[cfg(feature = "logging")]
use tracing::info;

/// Config for the Minecraft server.
#[derive(serde::Deserialize)]
pub struct MinecraftConfig {
    pub versions: HashMap<String, HashMap<String, LoaderConfig>>,
}

/// Config for the Minecraft server.
#[derive(serde::Deserialize)]
pub struct LoaderConfig {
    pub url: String,
    pub java_version: String,
    pub loader: String,
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
/// * `mc_config` - The Minecraft configuration file loaded as MinecraftConfig struct.
pub async fn download_server(
    name: &str,
    loader: &str,
    version: &str,
    agree_eula: bool,
    force_java_version: Option<String>,
    current_dir: PathBuf,
    mc_config: MinecraftConfig,
) -> Result<(String, String), SubstrateError> {
    if !agree_eula {
        return Err(SubstrateError::Eula);
    }

    let servers_path = current_dir.join("servers");
    let server_dir = servers_path.join(name);

    if tokio::fs::try_exists(&server_dir).await? {
        return Err(SubstrateError::AlreadyExists {
            resource: "Server with name already exists".to_string(),
        });
    }

    tokio::fs::create_dir_all(&server_dir).await?;

    let eula_path = server_dir.join("eula.txt");

    tokio::fs::write(eula_path, "eula=true").await?;

    let loader_config = mc_config
        .versions
        .get(version)
        .and_then(|loaders| loaders.get(loader))
        .ok_or(SubstrateError::NotFound {
            resource: "Could not find right minecraft version".to_string(),
        })?;

    let server_jar = server_dir.join("server.jar");

    download_helper(
        &loader_config.url,
        server_jar.to_str().ok_or(SubstrateError::ConversionError {
            details: "Failed to convert server jar path".to_string(),
        })?,
    )
    .await?;

    let java_version = force_java_version.unwrap_or_else(|| loader_config.java_version.clone());

    if !check_java(&java_version, &current_dir).await? {
        download_java(&java_version, current_dir).await?;
    }

    let server_config = server_dir.join("config.json");

    #[cfg(feature = "logging")]
    info!("Creating server config file");

    let config = serde_json::json!({
        "minecraft_version": version,
        "loader": loader,
        "java_version": java_version,
    });

    tokio::fs::write(server_config, serde_json::to_string_pretty(&config)?).await?;

    Ok((name.to_string(), java_version))
}
