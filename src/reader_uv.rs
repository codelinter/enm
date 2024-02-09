use crate::app_config::AppConfig;
use crate::user_version::UserVersion;
use crate::version_files::{get_user_version_for_directory, get_user_version_for_file};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum ReaderUV {
    Direct(UserVersion),
    Path(PathBuf),
}

impl ReaderUV {
    pub fn into_user_version(self, config: &AppConfig) -> Option<UserVersion> {
        match self {
            Self::Direct(uv) => Some(uv),
            Self::Path(pathbuf) if pathbuf.is_file() => get_user_version_for_file(pathbuf),
            Self::Path(pathbuf) => get_user_version_for_directory(pathbuf, config),
        }
    }
}

impl FromStr for ReaderUV {
    type Err = node_semver::SemverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pathbuf = PathBuf::from_str(s);
        let user_version = UserVersion::from_str(s);
        match (user_version, pathbuf) {
            (_, Ok(pathbuf)) if pathbuf.exists() => Ok(Self::Path(pathbuf)),
            (Ok(user_version), _) => Ok(Self::Direct(user_version)),
            (Err(user_version_err), _) => Err(user_version_err),
        }
    }
}
