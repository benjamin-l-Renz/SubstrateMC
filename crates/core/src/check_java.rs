use std::path::Path;

use crate::errors::error::SubstrateError;

#[cfg(feature = "logging")]
use tracing::info;

/// Checks if Java is installed and available in the `runtime` directory.
///
/// # Arguments
/// * `version` - The version of Java to check for.
/// * `current_dir` - The current project directory.
pub async fn check_java(version: &str, current_dir: &Path) -> Result<bool, SubstrateError> {
    #[cfg(feature = "logging")]
    info!("Checking if Java {} exists", &version);
    let java = current_dir
        .join("runtime")
        .join(format!("java-{}", version))
        .exists();
    Ok(java)
}
