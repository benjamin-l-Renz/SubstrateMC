use std::{collections::HashMap, path::PathBuf};

use crate::{
    check_java::check_java, download::download, download_java::download_java,
    errors::api_error::ApiError,
};

#[cfg(feature = "logging")]
use tracing::info;

#[derive(serde::Deserialize)]
pub struct MinecraftConfig {
    pub versions: HashMap<String, HashMap<String, LoaderConfig>>,
}

#[derive(serde::Deserialize)]
pub struct LoaderConfig {
    pub url: String,
    pub java_version: String,
    pub loader: String,
}

pub async fn download_server<'a>(
    name: &'a str,
    loader: &'a str,
    version: &'a str,
    agree_eula: bool,
    force_java_version: Option<&'a str>,
    current_dir: PathBuf,
) -> Result<&'a str, ApiError> {
    if !agree_eula {
        return Err(ApiError::InternalServerError);
    }

    let servers_path = current_dir.join("servers");
    let server_dir = servers_path.join(name);

    if tokio::fs::try_exists(&server_dir).await? {
        return Err(ApiError::InternalServerError);
    }

    tokio::fs::create_dir_all(&server_dir).await?;

    let eula_path = server_dir.join("eula.txt");

    tokio::fs::write(eula_path, "eula=true").await?;

    let config_dir = current_dir.join("minecraft.json");

    if !tokio::fs::try_exists(&config_dir).await? {
        return Err(ApiError::NotFound(
            "Could not find the minecraft.json".to_string(),
        ));
    }

    let config_data = tokio::fs::read_to_string(config_dir).await?;

    let mc_config: MinecraftConfig = serde_json::from_str(&config_data)?;

    let loader_config = mc_config
        .versions
        .get(version)
        .and_then(|loaders| loaders.get(loader))
        .ok_or(ApiError::NotFound(
            "Could not find right minecraft version".to_string(),
        ))?;

    let server_jar = server_dir.join("server.jar");

    download(
        &loader_config.url,
        server_jar.to_str().ok_or(ApiError::InternalServerError)?,
    )
    .await?;

    let java_version = force_java_version.unwrap_or(&loader_config.java_version);

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

    Ok(name)
}
