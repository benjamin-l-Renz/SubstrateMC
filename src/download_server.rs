use actix_web::web;
#[cfg(feature = "logging")]
use tracing::{error, info, warn};

use crate::{
    api::open_socket::SharedServers,
    check_java::check_java_installed,
    download::download,
    download_java::download_java_locally,
    errors::error::ApiError,
    indices::{add_entry, write_index_to_file},
    server::McServer,
};
use std::collections::HashMap;

#[derive(serde::Deserialize)]
pub struct MinecraftConfig {
    pub versions: HashMap<String, HashMap<String, LoaderConfig>>,
}

#[derive(serde::Deserialize)]
pub struct LoaderConfig {
    pub url: String,
    pub java_version: String,
}

pub async fn download_server(
    name: &str,
    mc_version: &str,
    mc_loader: &str,
    force_java_version: Option<&str>,
    data: web::Data<SharedServers>,
    agree_eula: bool,
) -> Result<u32, ApiError> {
    let project_dir = std::env::current_dir()?;
    let server_dir = project_dir.join("servers").join(name);

    if !agree_eula {
        return Err(ApiError::InternalServerError);
    }

    if !server_dir.exists() {
        tokio::fs::create_dir_all(&server_dir).await?;

        let eula_path = server_dir.join("eula.txt");

        tokio::fs::write(&eula_path, "eula=true").await?;

        #[cfg(feature = "logging")]
        info!("Server directory created at {:?}", server_dir);

        let count = std::fs::read_dir(project_dir.join("servers"))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .count();

        info!("Server count: {}", count);

        let index: u32 = count as u32;

        #[cfg(feature = "logging")]
        info!("reading index.bin file");
        let index_file = project_dir.join("servers").join("index.bin");

        if !index_file.exists() {
            tokio::fs::create_dir_all(&index_file).await?;
        }

        add_entry(index, name.to_string())?;

        write_index_to_file(&index_file)?;

        #[cfg(feature = "logging")]
        info!("Add server to list");
        data.write().await.servers.insert(index, McServer::new());

        #[cfg(feature = "logging")]
        info!("finding mc config file");
        let config_dir = project_dir.join("minecraft.json");

        if !tokio::fs::try_exists(&config_dir).await? {
            #[cfg(feature = "logging")]
            error!(
                "Minecraft configuration file not found (please create a minecraft.json file in the project directory)"
            );
            return Err(ApiError::NotFound);
        }
        #[cfg(feature = "logging")]
        info!("Reading mc config file");

        let config_data = tokio::fs::read_to_string(config_dir).await?;

        let mc_config: MinecraftConfig = serde_json::from_str(&config_data)?;

        #[cfg(feature = "logging")]
        info!("Find matching mc version and loader");

        let loader_config = mc_config
            .versions
            .get(mc_version)
            .and_then(|loaders| loaders.get(mc_loader))
            .ok_or(ApiError::NotFound)?;

        #[cfg(feature = "logging")]
        info!("Downloading server from {}...", loader_config.url);

        let server_jar = server_dir.join("server.jar");

        download(
            &loader_config.url,
            server_jar.to_str().ok_or(ApiError::InternalServerError)?,
        )
        .await?;

        #[cfg(feature = "logging")]
        info!("Checkking java installation");

        let java_version = force_java_version.unwrap_or(&loader_config.java_version);

        if !check_java_installed(java_version) {
            #[cfg(feature = "logging")]
            warn!("Java version not installed, installing...");
            download_java_locally(java_version).await?;
        }

        #[cfg(feature = "logging")]
        info!("Using Java version: {}", java_version);

        let server_config = server_dir.join("config.json");

        if !tokio::fs::try_exists(&server_config).await? {
            #[cfg(feature = "logging")]
            info!("Creating server config file");

            let config = serde_json::json!({
                "minecraft_version": mc_version,
                "loader": mc_loader,
                "java_version": java_version,
            });

            tokio::fs::write(server_config, serde_json::to_string_pretty(&config)?).await?;
        }

        return Ok(index);
    }

    #[cfg(feature = "logging")]
    error!(
        "Server directory already exists at {:?}. You can ignore this error if an server with that name already exists.",
        server_dir
    );
    Err(ApiError::InternalServerError)
}
