use super::{all::Error, Extract};
use std::{io::Read, path::Path};

pub enum Tape<R: Read> {
    /// XZ compression applied for Tape
    Xz(R),
    /// Gzip compression applied for Tape
    Gz(R),
}

impl<R: Read> Tape<R> {
    fn extract_into_impl<P: AsRef<Path>>(self, path: P) -> Result<(), Error> {
        let stream: Box<dyn Read> = match self {
            Self::Xz(response) => Box::new(xz2::read::XzDecoder::new(response)),
            Self::Gz(response) => Box::new(flate2::read::GzDecoder::new(response)),
        };
        let mut tar_pressed = tar::Archive::new(stream);
        tar_pressed.unpack(&path)?;
        Ok(())
    }
}

impl<R: Read> Extract for Tape<R> {
    fn extract_into(self: Box<Self>, path: &Path) -> Result<(), Error> {
        self.extract_into_impl(path)
    }
}
