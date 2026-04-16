use std::{future::Future, path::Path};

use crate::{download::server::installer::create_script::JavaFlags, errors::error::SubstrateError};

pub trait ServerInstaller {
    fn install(
        &self,
        server_dir: &Path,
        download_url: &str,
        java_version: &str,
        flags: &JavaFlags,
    ) -> impl Future<Output = Result<(), SubstrateError>> + Send;
}
