use crate::errors::error::ApiError;

pub async fn get_config(file_name: &str, server_name: &str) -> Result<String, ApiError> {
    let file_path = std::env::current_dir()?
        .join("servers")
        .join(server_name)
        .join(file_name);

    if !tokio::fs::try_exists(&file_path).await? {
        return Err(ApiError::NotFound);
    }

    let content = tokio::fs::read_to_string(file_path).await?;

    Ok(content)
}
