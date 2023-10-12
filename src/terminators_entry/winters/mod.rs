use super::terminators::Shell;
use std::path::Path;

#[derive(Debug)]
pub struct WinterX;

impl Shell for WinterX {
    fn to_clap_shell(&self) -> clap_complete::Shell {
        // TODO: move to Option
        panic!("Shell completion may only be supported for Windows PowerShell. Experimental");
    }

    fn path(&self, path: &Path) -> anyhow::Result<String> {
        let current_path =
            std::env::var_os("path").ok_or_else(|| anyhow::anyhow!("Unable to read PATH env var"))?;
        let mut split_paths: Vec<_> = std::env::split_paths(&current_path).collect();
        split_paths.insert(0, path.to_path_buf());
        let new_path = std::env::join_paths(split_paths)
            .map_err(|err| anyhow::anyhow!("Unable to join paths: {}", err))?;
        let new_path = new_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Unable to convert path to string"))?;
        Ok(format!("SET PATH={new_path}"))
    }

    fn set_env_var(&self, name: &str, value: &str) -> String {
        format!("SET {name}={value}")
    }

    fn on_enter(&self, config: &crate::app_config::AppConfig) -> anyhow::Result<String> {
        let path = config.base_dir_with_default().join("cd.cmd");
        create_cd_file_at(&path).map_err(|source| {
            anyhow::anyhow!(
                "Unable to create cd.cmd file for on-enter at {}: {}",
                path.display(),
                source
            )
        })?;
        let path = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Unable to read path to cd.cmd"))?;
        Ok(format!("doskey cd={path} $*",))
    }
}

fn create_cd_file_at(path: &std::path::Path) -> std::io::Result<()> {
    use std::io::Write;
    let cmd_contents = include_bytes!("./cd.cmd");
    let mut file = std::fs::File::create(path)?;
    file.write_all(cmd_contents)?;
    Ok(())
}