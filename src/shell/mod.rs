mod bash;
mod shell_detector;
mod powershell;
mod windows_cmd;
mod zsh;

#[allow(clippy::module_inception)]
mod shell;
mod windows_compat;

pub use bash::Bash;
pub use shell_detector::shell_detector;
pub use powershell::PowerShell;
pub use shell::{Shell, Shells};
pub use windows_cmd::WindowsCmd;
pub use windows_compat::maybe_fix_windows_path;
pub use zsh::Zsh;
