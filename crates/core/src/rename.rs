use crate::errors::api_error::ApiError;
#[cfg(feature = "logging")]
use tracing::info;

pub async fn rename_unpacked_java_folder(
    target_path: &std::path::PathBuf,
    target_name: &str,
    runtime_dir: &std::path::PathBuf,
) -> Result<(), ApiError> {
    // Return early if target already exists
    if tokio::fs::try_exists(target_path).await? {
        return Err(ApiError::InternalServerError);
    }

    let mut entries = tokio::fs::read_dir(runtime_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Get folder name
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if n != target_name => n,
            _ => continue,
        };

        // Check if folder looks like a JDK
        if name.to_uppercase().contains("JDK") {
            #[cfg(feature = "logging")]
            info!("Renaming {} to {}", name, target_name);

            tokio::fs::rename(&path, target_path).await?;
            break;
        }
    }

    Ok(())
}
