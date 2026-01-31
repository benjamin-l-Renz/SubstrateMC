use once_cell::sync::Lazy;

#[cfg(feature = "logging")]
use tracing::error;

use crate::download_java::JavaConfig;

/// caches java config so it is only read once
static JAVA_CONFIG: Lazy<JavaConfig> = Lazy::new(|| {
    match serde_json::from_str(
        &std::fs::read_to_string(
            std::env::current_dir()
                .expect("Failed to get current directory")
                .join("java.json"),
        )
        .expect("Failed to read java.json file"),
    ) {
        Ok(string) => string,
        Err(e) => {
            #[cfg(feature = "logging")]
            error!("Failed to parse java.json: {}", e);
            JavaConfig::default()
        }
    }
});

/// Checks if the Java runtime of the specified version is installed.
/// Thats posible by reading the json config and checking if the corresponding folder exists in the runtime directory.
pub fn check_java_installed(version: &str) -> bool {
    let project_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            #[cfg(feature = "logging")]
            error!("Failed to get current directory: {}", e);
            return false;
        }
    };

    let runtime_dir = project_dir.join("runtime");

    JAVA_CONFIG.linux.iter().any(|java| {
        java.version == version && runtime_dir.join(format!("java-{}", java.version)).exists()
    })
}
