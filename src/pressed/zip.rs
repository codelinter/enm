use super::all::{Error, Extract};
use log::debug;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use tempfile::tempfile;
use zip::read::ZipArchive;

pub struct Zip<R: Read> {
    response: R,
}

impl<R: Read> Zip<R> {
    #[allow(dead_code)]
    pub fn new(response: R) -> Self {
        Self { response }
    }
}

impl<R: Read> Extract for Zip<R> {
    fn extract_into(mut self: Box<Self>, path: &Path) -> Result<(), Error> {
        let mut tmp_zip_file = tempfile().expect("Unable to get a temporary file");

        debug!("Created a temporary zip file");
        io::copy(&mut self.response, &mut tmp_zip_file)?;
        debug!(
            "Wrote zipfile successfully. Now extracting into {}.",
            path.display()
        );

        let mut pressed = ZipArchive::new(&mut tmp_zip_file)?;

        for i in 0..pressed.len() {
            let mut file = pressed.by_index(i)?;
            let outpath = path.join(file.mangled_name());

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    debug!("File {} comment: {}", i, comment);
                }
            }

            if file.name().ends_with('/') {
                debug!(
                    "File {} extracted to \"{}\"",
                    i,
                    outpath.as_path().display()
                );
                fs::create_dir_all(&outpath)?;
            } else {
                debug!(
                    "Extracting file {} to \"{}\" ({} bytes)",
                    i,
                    outpath.as_path().display(),
                    file.size()
                );
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }

        Ok(())
    }
}
