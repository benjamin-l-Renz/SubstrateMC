use std::{collections::HashMap, path::Path};

#[cfg(feature = "logging")]
use tracing::warn;

#[derive(serde::Deserialize)]
struct MinecraftConfig {
    versions: HashMap<String, HashMap<String, LoaderConfig>>,
}

#[derive(serde::Deserialize)]
struct LoaderConfig {
    url: String,
    java_version: String,
}

pub struct ServerConfig<'a> {
    pub name: &'a str,
    pub loader: &'a str,
    pub version: &'a str,
}

use crate::{
    check_java::check_java,
    download::{
        download_java::download_java,
        server::installer::{create_script::JavaFlags, server_installer::ServerInstaller},
    },
    errors::error::SubstrateError,
};

pub async fn download_server<'a, T>(
    server_config: ServerConfig<'a>,
    agree_eula: bool,
    force_java_version: Option<String>,
    current_dir: &Path,
    flags: &JavaFlags,
    installer: T,
) -> Result<(String, String), SubstrateError>
where
    T: ServerInstaller,
{
    if !agree_eula {
        return Err(SubstrateError::Eula);
    }

    let servers_dir = &current_dir.join("servers");

    if !tokio::fs::try_exists(servers_dir).await? {
        #[cfg(feature = "logging")]
        warn!("Servers directory does not exist");

        tokio::fs::create_dir_all(&servers_dir).await?;
    }

    let server_path = &servers_dir.join(server_config.name);

    if tokio::fs::try_exists(server_path).await? {
        return Err(SubstrateError::AlreadyExists {
            resource: "Directory with server name does already exist".to_string(),
        });
    }

    tokio::fs::create_dir(server_path).await?;

    // Create eula file and agree
    let eula_dir = &server_path.join("eula.txt");

    tokio::fs::write(&eula_dir, "eula=true").await?;

    let config = get_config(current_dir, &server_config).await?;

    let java_version = force_java_version.unwrap_or(config.java_version);

    if !check_java(&java_version, current_dir).await? {
        download_java(&java_version, current_dir).await?;
    }

    // TODO: install server over installer

    installer
        .install(server_path, &config.url, &java_version, flags)
        .await?;

    Ok((server_config.name.to_string(), java_version))
}

async fn get_config<'a>(
    current_dir: &Path,
    server_config: &ServerConfig<'a>,
) -> Result<LoaderConfig, SubstrateError> {
    let config_dir = &current_dir.join("minecraft.json");

    let config_str = tokio::fs::read_to_string(&config_dir).await?;

    let mut config: MinecraftConfig = serde_json::from_str(&config_str)?;

    let mut inner = match config.versions.remove(server_config.version) {
        Some(v) => v,
        None => {
            return Err(SubstrateError::NotFound {
                resource: "Could not find minecraft version in minecraft.json".to_string(),
            });
        }
    };

    Ok(match inner.remove(server_config.loader) {
        Some(v) => v,
        None => {
            return Err(SubstrateError::NotFound {
                resource: "Could not find minecraft loader in minecraft.json".to_string(),
            });
        }
    })
}
