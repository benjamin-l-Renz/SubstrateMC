use std::path::Path;

use crate::{
    download::{
        download_helper::download_helper,
        server::installer::create_script::{JavaFlags, create_server_script},
    },
    errors::error::SubstrateError,
};

pub mod vanilla;

async fn download_without_installer(
    server_dir: &Path,
    url: &str,
    flags: &JavaFlags,
    java_version: &str,
) -> Result<(), SubstrateError> {
    let server_jar = &server_dir.join("server.jar");

    download_helper(url, server_jar).await?;

    create_server_script(server_dir, flags, java_version).await?;

    Ok(())
}

async fn download_with_installer(
    server_dir: &Path,
    url: &str,
    flags: &JavaFlags,
    java_version: &str,
) -> Result<(), SubstrateError> {
    Ok(())
}
