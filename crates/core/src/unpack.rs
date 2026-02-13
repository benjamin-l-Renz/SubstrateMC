use crate::errors::api_error::ApiError;

/// unpackes a given archive
///
/// # Errors
///
/// This function will return an error if the function isnt able to open the archive.
pub async fn unpack(archive_path: &str, extract_path: &str) -> Result<(), ApiError> {
    #[cfg(target_os = "windows")]
    {
        println!("Unpacking is not supported on Windows yet.");
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    untar(archive_path, extract_path)?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn untar(archive_path: &str, extract_path: &str) -> Result<(), ApiError> {
    let file = std::fs::File::open(archive_path)?;

    let decompresser = flate2::read::GzDecoder::new(file);

    let mut archive = tar::Archive::new(decompresser);

    archive.unpack(extract_path)?;

    Ok(())
}
