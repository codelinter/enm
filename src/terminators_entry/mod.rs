mod bash;
mod conjectr;
mod pshell;
mod winters;
mod zsh;

#[allow(clippy::module_inception)]
mod terminators;
mod mic_patch;

pub use bash::Bash;
pub use conjectr::conjectr_shell;
pub use pshell::PowerShell;
pub use terminators::{Shell, Terms};
pub use mic_patch::microsoft_prod_patch_path;
pub use winters::WinterX;
pub use zsh::Zsh;
