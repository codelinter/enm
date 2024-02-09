use clap::ValueEnum;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum SiftMethod {
    /// Look for NodeJS version in the current directory
    #[default]
    Local,
    /// Look for NodeJS version in the current directory. If not found, moving up to parent folders
    Recursive,
}

impl SiftMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            SiftMethod::Local => "local",
            SiftMethod::Recursive => "recursive",
        }
    }
}

impl Display for SiftMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
