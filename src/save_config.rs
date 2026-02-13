use crate::errors::error::ApiError;

pub async fn save_config(
    server_name: &str,
    file_name: &str,
    file_content: &str,
) -> Result<(), ApiError> {
    let file_path = std::env::current_dir()?.join(server_name).join(file_name);

    if !tokio::fs::try_exists(&file_path).await? {
        return Err(ApiError::NotFound);
    }

    tokio::fs::write(file_path, file_content).await?;

    Ok(())
}
