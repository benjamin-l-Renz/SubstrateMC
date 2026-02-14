use crate::download::download_server::download_server;
use std::path::PathBuf;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn create_server(
    name: *const std::os::raw::c_char,
    loader: *const std::os::raw::c_char,
    version: *const std::os::raw::c_char,
    agree_eula: bool,
    force_java_version: *const std::os::raw::c_char,
) {
    use std::ffi::CStr;

    let name = unsafe { CStr::from_ptr(name).to_str().unwrap() };
    let loader = unsafe { CStr::from_ptr(loader).to_str().unwrap() };
    let version = unsafe { CStr::from_ptr(version).to_str().unwrap() };
    let force_java_version = if force_java_version.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(force_java_version).to_str().unwrap() })
    };

    let current_dir = std::env::current_dir().expect("Failed to get current working directory");

    // Create a new runtime and block on the async function
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(download_server(
        name,
        loader,
        version,
        agree_eula,
        force_java_version,
        current_dir,
    ));
}
