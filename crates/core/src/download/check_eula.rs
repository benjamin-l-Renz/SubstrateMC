use std::path::Path;

use crate::errors::error::SubstrateError;

pub async fn check_eula(server_dir: &Path) -> Result<bool, SubstrateError> {
    let content = tokio::fs::read_to_string(server_dir.join("eula.txt")).await?;

    if content.contains("eula=true") {
        return Ok(true);
    }

    Ok(false)
}

pub async fn create_eula(server_dir: &Path) -> Result<(), SubstrateError> {
    let eula_dir = server_dir.join("eula.txt");

    if !eula_dir.exists() {
        tokio::fs::write(eula_dir, "").await?;
    } else {
        return Err(SubstrateError::AlreadyExists {
            resource: "Eula file is already existing".to_string(),
        });
    }

    Ok(())
}

pub async fn agree_eula(server_dir: &Path) -> Result<(), SubstrateError> {
    let eula_dir = server_dir.join("eula.txt");

    tokio::fs::write(eula_dir, "eula=true").await?;

    Ok(())
}
