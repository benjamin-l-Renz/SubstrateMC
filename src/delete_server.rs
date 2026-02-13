use crate::errors::error::ApiError;

pub async fn delete_server(server_name: &str) -> Result<(), ApiError> {
    let project_dir = std::env::current_dir()?;
    let server_dir = project_dir.join("servers").join(server_name);

    if !tokio::fs::try_exists(&server_dir).await? {
        return Err(ApiError::NotFound);
    }

    tokio::fs::remove_dir_all(server_dir).await?;
    Ok(())
}
