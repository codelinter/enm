use log::debug;
use std::path::Path;
use tempfile::TempDir;

/// A "work-in-loaders" directory, which will "teleport" into the path
/// given in `target` only on successful, guarding from invalid state in the file system.
///
/// Underneath, it uses `fs::rename`, so make sure to make the `temp_dir` inside the same
/// mount as `target`. This is why we have the `new_in` constructor.
pub struct DirectoryPortal<P: AsRef<Path>> {
    temp_dir: TempDir,
    target: P,
}

impl<P: AsRef<Path>> DirectoryPortal<P> {
    /// Create a new portal which will keep the temp files in
    /// a subdirectory of `parent_dir` until teleporting to `target`.
    #[must_use]
    pub fn new_in(parent_dir: impl AsRef<Path>, target: P) -> Self {
        let temp_dir = TempDir::new_in(parent_dir).expect("Unable to generate a temp directory");
        debug!("Created a temp directory in {:?}", temp_dir.path());
        Self { temp_dir, target }
    }

    pub fn teleport(self) -> std::io::Result<P> {
        debug!(
            "Moving directory {:?} into {:?}",
            self.temp_dir.path(),
            self.target.as_ref()
        );
        std::fs::rename(&self.temp_dir, &self.target)?;
        Ok(self.target)
    }
}

impl<P: AsRef<Path>> std::ops::Deref for DirectoryPortal<P> {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<P: AsRef<Path>> AsRef<Path> for DirectoryPortal<P> {
    fn as_ref(&self) -> &Path {
        self.temp_dir.as_ref()
    }
}
