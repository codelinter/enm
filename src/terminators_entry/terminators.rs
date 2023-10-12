use std::fmt::{Debug, Display};
use std::path::Path;

use clap::ValueEnum;

pub trait Shell: Debug {
    fn path(&self, path: &Path) -> anyhow::Result<String>;
    fn set_env_var(&self, name: &str, value: &str) -> String;
    fn on_enter(&self, config: &crate::app_config::AppConfig) -> anyhow::Result<String>;
    fn to_clap_shell(&self) -> clap_complete::Shell;
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Terms {
    Bash,
    Zsh,
    #[clap(name = "powershell", alias = "pshell")]
    PowerShell,
    #[cfg(windows)]
    Cmd,
}

impl Display for Terms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Terms::Bash => f.write_str("bash"),
            Terms::Zsh => f.write_str("zsh"),
            Terms::PowerShell => f.write_str("powershell"),
            #[cfg(windows)]
            Terms::Cmd => f.write_str("cmd"),
        }
    }
}

impl From<Terms> for Box<dyn Shell> {
    fn from(shell: Terms) -> Box<dyn Shell> {
        match shell {
            Terms::Zsh => Box::from(super::zsh::Zsh),
            Terms::Bash => Box::from(super::bash::Bash),
            Terms::PowerShell => Box::from(super::pshell::PowerShell),
            #[cfg(windows)]
            Terms::Cmd => Box::from(super::winters::WinterX),
        }
    }
}

impl From<Box<dyn Shell>> for clap_complete::Shell {
    fn from(shell: Box<dyn Shell>) -> Self {
        shell.to_clap_shell()
    }
}
