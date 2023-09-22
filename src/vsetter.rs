use crate::config::EnmConfig;
use crate::fs::{remove_symlink_dir, shallow_read_symlink, symlink_dir};
use crate::system_version;
use crate::version::Version;
use std::convert::TryInto;
use std::path::PathBuf;

pub fn create_vsetter(
    config: &EnmConfig,
    common_name: &str,
    version: &Version,
) -> std::io::Result<()> {
    let vsetteres_dir = config.vsetteres_dir();
    std::fs::create_dir_all(&vsetteres_dir)?;

    let version_dir = version.installation_path(config);
    let vsetter_dir = vsetteres_dir.join(common_name);

    remove_symlink_dir(&vsetter_dir).ok();
    symlink_dir(version_dir, &vsetter_dir)?;

    Ok(())
}

pub fn list_aliases(config: &EnmConfig) -> std::io::Result<Vec<StoredVSetter>> {
    let vec: Vec<_> = std::fs::read_dir(config.vsetteres_dir())?
        .filter_map(Result::ok)
        .filter_map(|x| TryInto::<StoredVSetter>::try_into(x.path().as_path()).ok())
        .collect();
    Ok(vec)
}

#[derive(Debug)]
pub struct StoredVSetter {
    vsetter_path: PathBuf,
    destination_path: PathBuf,
}

impl std::convert::TryInto<StoredVSetter> for &std::path::Path {
    type Error = std::io::Error;

    fn try_into(self) -> Result<StoredVSetter, Self::Error> {
        let shallow_self = shallow_read_symlink(self)?;
        let destination_path = if shallow_self == system_version::path() {
            shallow_self
        } else {
            std::fs::canonicalize(&shallow_self)?
        };
        Ok(StoredVSetter {
            vsetter_path: PathBuf::from(self),
            destination_path,
        })
    }
}

impl StoredVSetter {
    pub fn s_ver(&self) -> &str {
        if self.destination_path == system_version::path() {
            system_version::display_name()
        } else {
            self.destination_path
                .parent()
                .unwrap()
                .file_name()
                .expect("must have basename")
                .to_str()
                .unwrap()
        }
    }

    pub fn name(&self) -> &str {
        self.vsetter_path
            .file_name()
            .expect("must have basename")
            .to_str()
            .unwrap()
    }

    pub fn path(&self) -> &std::path::Path {
        &self.vsetter_path
    }
}
