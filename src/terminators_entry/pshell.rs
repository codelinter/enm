use crate::sift_method::SiftMethod;

use super::Shell;
use indoc::formatdoc;
use std::path::Path;

#[derive(Debug)]
pub struct PowerShell;

impl Shell for PowerShell {
    fn path(&self, path: &Path) -> anyhow::Result<String> {
        let current_path =
            std::env::var_os("PATH").ok_or_else(|| anyhow::anyhow!("Unable to read PATH env var"))?;
        let mut split_paths: Vec<_> = std::env::split_paths(&current_path).collect();
        split_paths.insert(0, path.to_path_buf());
        let new_path = std::env::join_paths(split_paths)
            .map_err(|source| anyhow::anyhow!("Unable to join paths: {}", source))?;
        let new_path = new_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Unable to read PATH"))?;
        Ok(self.set_env_var("PATH", new_path))
    }

    fn set_env_var(&self, name: &str, value: &str) -> String {
        format!(r#"$env:{name} = "{value}""#)
    }

    fn on_enter(&self, config: &crate::app_config::AppConfig) -> anyhow::Result<String> {
        let version_file_exists_condition = String::from("(Test-Path .nvmrc) -Or (Test-Path .node-version)").to_string();
        
        let trigger_autoload = match config.sift_method() {
            SiftMethod::Local => formatdoc!(
                r#"
                    If ({version_file_exists_condition}) {{ & enm switch --caps-lock-when-needed }}
                "#,
                version_file_exists_condition = version_file_exists_condition,
            ),
            SiftMethod::Recursive => String::from(r"enm switch --caps-lock-when-needed"),
        };
        Ok(formatdoc!(
            r#"
                function global:Set-EnmOnLoad {{ {trigger_autoload} }}
                function global:Set-LocationWithEnm {{ param($path); if ($path -eq $null) {{Set-Location}} else {{Set-Location $path}}; Set-EnmOnLoad }}
                Set-Alias -Scope global cd_with_fnm Set-LocationWithEnm
                Set-Alias -Option AllScope -Scope global cd Set-LocationWithEnm
                Set-EnmOnLoad
            "#,
            trigger_autoload = trigger_autoload
        ))
    }
    fn to_clap_shell(&self) -> clap_complete::Shell {
        clap_complete::Shell::PowerShell
    }
}
