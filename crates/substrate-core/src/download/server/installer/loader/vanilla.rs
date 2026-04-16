use crate::download::server::installer::{
    create_script::JavaFlags, loader::download_without_installer, server_installer::ServerInstaller,
};

pub struct VanillaInstaller {}

impl ServerInstaller for VanillaInstaller {
    async fn install(
        &self,
        server_dir: &std::path::Path,
        download_url: &str,
        java_version: &str,
        flags: &JavaFlags,
    ) -> Result<(), crate::errors::error::SubstrateError> {
        download_without_installer(server_dir, download_url, flags, java_version).await?;

        Ok(())
    }
}
