use std::path::Path;

use crate::errors::api_error::ApiError;

pub async fn delete_server(name: &str, current_dir: &Path) -> Result<(), ApiError> {
    let server_dir = current_dir.join("servers").join(name);

    if !tokio::fs::try_exists(&server_dir).await? {
        return Err(ApiError::NotFound(
            "Could not find server directory".to_string(),
        ));
    }

    tokio::fs::remove_dir_all(server_dir).await?;

    Ok(())
}
