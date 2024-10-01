use crate::cd_file_changer::VersionFileStrategy;

use super::shell::Shell;
use indoc::formatdoc;
use std::path::Path;

#[derive(Debug)]
pub struct Bash;

impl Shell for Bash {
    fn to_clap_shell(&self) -> clap_complete::Shell {
        clap_complete::Shell::Bash
    }

    fn path(&self, path: &Path) -> anyhow::Result<String> {
        let path = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Can't convert path to string"))?;
        let path =
            super::windows_compat::maybe_fix_windows_path(path).unwrap_or_else(|| path.to_string());
        Ok(format!("export PATH={path:?}:\"$PATH\""))
    }

    fn set_env_var(&self, name: &str, value: &str) -> String {
        format!("export {name}={value:?}")
    }

    fn detect_node(&self, config: &crate::config::EnmConfig) -> anyhow::Result<String> {
        let version_file_exists_condition = if config.resolve_engines() {
            "-f .node-version || -f .nvmrc || -f package.json"
        } else {
            "-f .node-version || -f .nvmrc"
        };
        let autoload_hook = match config.cd_file_changer() {
            VersionFileStrategy::Local => formatdoc!(
                r#"
                    if [[ {version_file_exists_condition} ]]; then
                        enm use --silent-if-unchanged
                    fi
                "#,
                version_file_exists_condition = version_file_exists_condition,
            ),
            VersionFileStrategy::Recursive => String::from(r"enm use --silent-if-unchanged"),
        };
        Ok(formatdoc!(
            r#"
                __enm_use_if_file_found() {{
                    {autoload_hook}
                }}

                __enmcd() {{
                    \cd "$@" || return $?
                    __enm_use_if_file_found
                }}

                vsetter cd=__enmcd
                __enm_use_if_file_found
            "#,
            autoload_hook = autoload_hook
        ))
    }
}
