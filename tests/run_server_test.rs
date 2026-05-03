use substrate_core::{
    download::server::{
        download_server::ServerConfig, installer::loader::vanilla::VanillaInstaller,
    },
    server::Server,
};
use tempfile::tempdir;

#[tokio::test]
async fn async_test_run_server() {
    let dir = tempdir().unwrap();

    let config_files = ["minecraft.json", "java.json"];

    for config in &config_files {
        let src = std::env::current_dir().unwrap().join(config);
        let dest = dir.path().join(config);
        std::fs::copy(src, &dest).expect("failed to copy config");
    }

    let config = ServerConfig {
        name: "test",
        loader: "vanilla",
        version: "1.21.11",
    };

    let (name, _) = substrate_core::download::server::download_server::download_server(
        config,
        true,
        None,
        dir.path(),
        &substrate_core::download::server::installer::create_script::JavaFlags::Normal,
        VanillaInstaller {},
    )
    .await
    .unwrap();

    let output_file = dir.path().join("servers").join("test").join("server.jar");

    assert!(output_file.exists());

    let java_installation = dir.path().join("runtime").join("java-21");

    assert!(java_installation.exists());

    let mut server = Server::new();

    server
        .start_server(&name, dir.path())
        .expect("Failed to start server");
}
