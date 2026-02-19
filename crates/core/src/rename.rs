use crate::errors::error::SubstrateError;
#[cfg(feature = "logging")]
use tracing::info;

/// Renames the unpacked Java folder to the target name.
///
/// Tries to rename the unpacked java folder by searching for a folder that looks like a JDK archive.
///
/// # Arguments
///
/// * `target_path` - The target path to rename the folder to.
/// * `target_name` - The target name to rename the folder to.
/// * `runtime_dir` - The runtime directory to search for the folder.
pub async fn rename_unpacked_java_folder(
    target_path: &std::path::PathBuf,
    target_name: &str,
    runtime_dir: &std::path::PathBuf,
) -> Result<(), SubstrateError> {
    // Return early if target already exists
    if tokio::fs::try_exists(target_path).await? {
        return Err(SubstrateError::AlreadyExists {
            resource: "Target path already exists".to_string(),
        });
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
