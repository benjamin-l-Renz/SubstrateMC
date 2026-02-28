use std::path::Path;

use crate::errors::error::SubstrateError;
#[cfg(feature = "logging")]
use tracing::{error, info};

/// Removes a server from the servers directory.
///
/// # Arguments
///
/// * `name` - The name of the server to remove.
/// * `current_dir` - The current root directory.
pub async fn delete_server(name: &str, current_dir: &Path) -> Result<(), SubstrateError> {
    #[cfg(feature = "logging")]
    info!("Deleting Server with name: {}", &name);
    let server_dir = current_dir.join("servers").join(name);

    if !tokio::fs::try_exists(&server_dir).await? {
        #[cfg(feature = "logging")]
        error!("Could not find server directory: ./servers/{}", &name);
        return Err(SubstrateError::NotFound {
            resource: "Could not find server directory".to_string(),
        });
    }

    tokio::fs::remove_dir_all(server_dir).await?;

    Ok(())
}
