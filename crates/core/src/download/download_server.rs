use std::{collections::HashMap, path::Path};

use crate::{
    check_java::check_java,
    download::{
        download_helper::download_helper, download_java::download_java,
        installer::server_installer::ServerInstaller,
    },
    errors::error::SubstrateError,
};

use tokio::process::Command;
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

#[derive(serde::Deserialize)]
pub enum JavaFlags {
    Aikars,
    Normal,
    Velocity,
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
pub async fn download_server<T>(
    name: &str,
    loader: &str,
    version: &str,
    agree_eula: bool,
    force_java_version: Option<String>,
    current_dir: &Path,
    flags: JavaFlags,
    installer: T,
) -> Result<(String, String), SubstrateError>
where
    T: ServerInstaller,
{
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
        warn!("Servers directory does not exist creating new one");
        tokio::fs::create_dir_all(&servers_directory).await?;
    }

    #[cfg(feature = "logging")]
    trace!("Joining server dir");
    let server_dir = &servers_directory.join(name);

    if tokio::fs::try_exists(server_dir).await? {
        return Err(SubstrateError::AlreadyExists {
            resource: "Directory with name already exists".to_string(),
        });
    }

    tokio::fs::create_dir_all(server_dir).await?;

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

    if !check_java(&java_version, current_dir).await? {
        #[cfg(feature = "logging")]
        info!("Java version not found installing...");
        download_java(&java_version, current_dir).await?;
    }

    if loader != "neoforge" && loader != "forge" {
        let server_jar = server_dir.join("server.jar");

        #[cfg(feature = "logging")]
        info!("Downloading server.jar");
        download_helper(&config.url, &server_jar).await?;

        let run_file = server_dir.join("run.sh");

        let contents = match flags {
            JavaFlags::Normal => format!(
                "#!/usr/bin/env bash\n../../runtime/java-{}/bin/java -jar server.jar nogui\n",
                java_version
            ),

            _ => {
                return Err(SubstrateError::NotFound {
                    resource: "other java flags are not supported yet".to_string(),
                });
            }
        };

        tokio::fs::write(&run_file, contents).await?;

        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut perms = tokio::fs::metadata(&run_file).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&run_file, perms).await?;
        }

        #[cfg(target_os = "windows")]
        {
            return Err(SubstrateError::NotFound {
                resource: "Bash files cant be run on windows".to_string(),
            });
        }

        #[cfg(feature = "logging")]
        trace!("Dropping server jar path");
        drop(server_jar);
    } else if loader == "neoforge" {
        let installer = server_dir.join("neoforge-installer.jar");

        download_helper(&config.url, &installer).await?;

        Command::new(format!("../../runtime/java-{}/bin/java", java_version))
            .arg("-jar")
            .arg("neoforge-installer.jar")
            .arg("--installServer")
            .current_dir(server_dir)
            .spawn()?
            .wait()
            .await?;
    } else if loader == "forge" {
        #[cfg(feature = "logging")]
        warn!("Forge support isnt stable yet");

        let installer = server_dir.join("forge-installer.jar");

        download_helper(&config.url, &installer).await?;

        Command::new(format!("../../runtime/java-{}/bin/java", java_version))
            .arg("-jar")
            .arg("forge-installer.jar")
            .arg("--installServer")
            .current_dir(server_dir)
            .spawn()?
            .wait()
            .await?;
    }

    Ok((name.to_string(), java_version))
}
