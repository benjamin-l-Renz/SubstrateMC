use std::path::Path;

use crate::errors::error::SubstrateError;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum JavaFlags {
    Aikars,
    Normal,
}

pub(crate) async fn create_server_script(
    server_dir: &Path,
    flags: &JavaFlags,
    java_version: &str,
) -> Result<(), SubstrateError> {
    let run_script = server_dir.join("run.sh");

    match flags {
        JavaFlags::Aikars => {
            tokio::fs::write(
            &run_script,
            format!(
                "#!/usr/bin/env bash\n../../runtime/java-{}/bin/java -XX:+AlwaysPreTouch -XX:+DisableExplicitGC -XX:+ParallelRefProcEnabled -XX:+PerfDisableSharedMem -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1HeapRegionSize=8M -XX:G1HeapWastePercent=5 -XX:G1MaxNewSizePercent=40 -XX:G1MixedGCCountTarget=4 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1NewSizePercent=30 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:MaxGCPauseMillis=200 -XX:MaxTenuringThreshold=1 -XX:SurvivorRatio=32 -Dusing.aikars.flags=https://mcflags.emc.gs -Daikars.new.flags=true -jar server.jar --nogui",
                java_version
            ),
        ).await?;
        }

        JavaFlags::Normal => {
            tokio::fs::write(
                &run_script,
                format!(
                    "#!/usr/bin/env bash\n../../runtime/java-{}/bin/java -jar server.jar --nogui\n",
                    java_version
                ),
            )
            .await?;
        }
    };

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = tokio::fs::metadata(&run_script).await?.permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&run_script, perms).await?;
    }

    Ok(())
}
