use std::path::Path;

use crate::errors::error::SubstrateError;

/// Removes a server from the servers directory.
///
/// # Arguments
///
/// * `name` - The name of the server to remove.
/// * `current_dir` - The current root directory.
pub async fn delete_server(name: &str, current_dir: &Path) -> Result<(), SubstrateError> {
    let server_dir = current_dir.join("servers").join(name);

    if !tokio::fs::try_exists(&server_dir).await? {
        return Err(SubstrateError::NotFound {
            resource: "Could not find server directory".to_string(),
        });
    }

    tokio::fs::remove_dir_all(server_dir).await?;

    Ok(())
}
