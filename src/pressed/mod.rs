pub mod all;
pub mod tape;
pub mod zip;

use std::io::Read;
use std::path::Path;

pub use self::all::{Error, Extract};
#[cfg(unix)]
use self::tape::Tape;

#[cfg(windows)]
use self::zip::Zip;

pub enum Archive {
    #[cfg(windows)]
    Zip,
    #[cfg(unix)]
    XzTape,
    #[cfg(unix)]
    GzTape,
}

impl Archive {
    pub fn extract_pressed_into(&self, path: &Path, response: impl Read) -> Result<(), Error> {
        let extractor: Box<dyn Extract> = match self {
            #[cfg(windows)]
            Self::Zip => Box::new(Zip::new(response)),
            #[cfg(unix)]
            Self::XzTape => Box::new(Tape::Xz(response)),
            #[cfg(unix)]
            Self::GzTape => Box::new(Tape::Gz(response)),
        };
        extractor.extract_into(path)?;
        Ok(())
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            #[cfg(windows)]
            Self::Zip => "zip",
            #[cfg(unix)]
            Self::XzTape => "tar.xz",
            #[cfg(unix)]
            Self::GzTape => "tar.gz",
        }
    }

    #[cfg(windows)]
    pub fn supported() -> &'static [Self] {
        &[Self::Zip]
    }

    #[cfg(unix)]
    pub fn supported() -> &'static [Self] {
        &[Self::XzTape, Self::GzTape]
    }
}
