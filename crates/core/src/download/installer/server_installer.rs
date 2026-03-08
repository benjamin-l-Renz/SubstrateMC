use std::path::Path;

use crate::{download::download_server::JavaFlags, errors::error::SubstrateError};

pub trait ServerInstaller {
    fn installer(
        &self,
        server_dir: &Path,
        download_url: &str,
        flags: JavaFlags,
    ) -> Result<(), SubstrateError>;
}
