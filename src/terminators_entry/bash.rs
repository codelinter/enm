use crate::sift_method::SiftMethod;

use super::terminators::Shell;
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
            .ok_or_else(|| anyhow::anyhow!("Unable to convert path to string"))?;
        let path =
            super::mic_patch::microsoft_prod_patch_path(path).unwrap_or_else(|| path.to_string());
        Ok(format!("export PATH={path:?}:\"$PATH\""))
    }

    fn set_env_var(&self, name: &str, value: &str) -> String {
        format!("export {name}={value:?}")
    }

    fn on_enter(&self, config: &crate::app_config::AppConfig) -> anyhow::Result<String> {
        let version_file_exists_condition =  String::from("-f .node-version || -f .nvmrc").to_string();
    
        let trigger_autoload = match config.sift_method() {
            SiftMethod::Local => formatdoc!(
                r#"
                    if [[ {version_file_exists_condition} ]]; then
                        enm switch --caps-lock-when-needed
                    fi
                "#,
                version_file_exists_condition = version_file_exists_condition,
            ),
            SiftMethod::Recursive => String::from(r"enm switch --caps-lock-when-needed"),
        };
        Ok(formatdoc!(
            r#"
                __enm_use_if_file_found() {{
                    {trigger_autoload}
                }}

                __enmcd() {{
                    \cd "$@" || return $?
                    __enm_use_if_file_found
                }}

                alias cd=__enmcd
                __enm_use_if_file_found
            "#,
            trigger_autoload = trigger_autoload
        ))
    }
}
