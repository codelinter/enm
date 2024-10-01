mod unix;

#[cfg(unix)]
pub use self::unix::shell_detector;
fn shell_from_string(shell: &str) -> Option<Box<dyn super::Shell>> {
    use super::{Bash, PowerShell, WindowsCmd, Zsh};
    match shell {
        "sh" | "bash" => return Some(Box::from(Bash)),
        "zsh" => return Some(Box::from(Zsh)),
        "powershell" => return Some(Box::from(PowerShell)),
        "cmd" => return Some(Box::from(WindowsCmd)),
        cmd_name => log::debug!("unsupported shell: {:?}", cmd_name),
    };
    None
}
