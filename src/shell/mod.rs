mod bash;
mod shell_detector;
mod zsh;

pub use bash::Bash;
pub use shell_detector::shell_detector;
pub use shell::{Shell, Shells};
pub use zsh::Zsh;
