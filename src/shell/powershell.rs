use crate::cd_file_changer::VersionFileStrategy;

use super::Shell;
use indoc::formatdoc;
use std::path::Path;

#[derive(Debug)]
pub struct PowerShell;

impl Shell for PowerShell {
    fn path(&self, path: &Path) -> anyhow::Result<String> {
        let current_path =
            std::env::var_os("PATH").ok_or_else(|| anyhow::anyhow!("Can't read PATH env var"))?;
        let mut split_paths: Vec<_> = std::env::split_paths(&current_path).collect();
        split_paths.insert(0, path.to_path_buf());
        let new_path = std::env::join_paths(split_paths)
            .map_err(|source| anyhow::anyhow!("Can't join paths: {}", source))?;
        let new_path = new_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Can't read PATH"))?;
        Ok(self.set_env_var("PATH", new_path))
    }

    fn set_env_var(&self, name: &str, value: &str) -> String {
        format!(r#"$env:{name} = "{value}""#)
    }

    fn detect_node(&self, config: &crate::config::EnmConfig) -> anyhow::Result<String> {
        let version_file_exists_condition = if config.resolve_engines() {
            "(Test-Path .nvmrc) -Or (Test-Path .node-version) -Or (Test-Path package.json)"
        } else {
            "(Test-Path .nvmrc) -Or (Test-Path .node-version)"
        };
        let autoload_hook = match config.cd_file_changer() {
            VersionFileStrategy::Local => formatdoc!(
                r#"
                    If ({version_file_exists_condition}) {{ & enm use --silent-if-unchanged }}
                "#,
                version_file_exists_condition = version_file_exists_condition,
            ),
            VersionFileStrategy::Recursive => String::from(r"enm use --silent-if-unchanged"),
        };
        Ok(formatdoc!(
            r#"
                function global:Set-EnmOnLoad {{ {autoload_hook} }}
                function global:Set-LocationWithEnm {{ param($path); if ($path -eq $null) {{Set-Location}} else {{Set-Location $path}}; Set-EnmOnLoad }}
                Set-VSetter -Scope global cd_with_enm Set-LocationWithEnm
                Set-VSetter -Option AllScope -Scope global cd Set-LocationWithEnm
                Set-EnmOnLoad
            "#,
            autoload_hook = autoload_hook
        ))
    }
    fn to_clap_shell(&self) -> clap_complete::Shell {
        clap_complete::Shell::PowerShell
    }
}
