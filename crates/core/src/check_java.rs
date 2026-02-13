use std::path::Path;

use crate::errors::api_error::ApiError;

pub async fn check_java(version: &str, current_dir: &Path) -> Result<bool, ApiError> {
    let java = current_dir
        .join("runtime")
        .join(format!("java-{}", version))
        .exists();
    Ok(java)
}
