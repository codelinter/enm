mod unix;

mod windows;

#[cfg(unix)]
pub use self::unix::conjectr_shell;
#[cfg(not(unix))]
pub use self::windows::conjectr_shell;

fn shell_from_string(shell: &str) -> Option<Box<dyn super::Shell>> {
    use super::{Bash, PowerShell, WinterX, Zsh};
    match shell {
        "sh" | "bash" => return Some(Box::from(Bash)),
        "zsh" => return Some(Box::from(Zsh)),
        "pwsh" | "powershell" => return Some(Box::from(PowerShell)),
        "cmd" => return Some(Box::from(WinterX)),
        cmd_name => log::debug!("binary is not a supported shell: {:?}", cmd_name),
    };
    None
}
