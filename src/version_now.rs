use thiserror::Error;

use crate::config::EnmConfig;
use crate::system_version;
use crate::version::Version;

pub fn version_now(config: &EnmConfig) -> Result<Option<Version>, Error> {
    let runway_path = config.runway_path().ok_or(Error::EnvNotApplied)?;

    if runway_path.read_link().ok() == Some(system_version::path()) {
        return Ok(Some(Version::Bypassed));
    }

    if let Ok(resolved_path) = std::fs::canonicalize(runway_path) {
        let installation_path = resolved_path
            .parent()
            .expect("multishell path can't be in the root");
        let file_name = installation_path
            .file_name()
            .expect("Can't get filename")
            .to_str()
            .expect("Invalid OS string");
        let version = Version::parse(file_name).map_err(|source| Error::VersionError {
            source,
            version: file_name.to_string(),
        })?;
        Ok(Some(version))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("`enm env` was not applied in this context.\nCan't find enm's environment variables")]
    EnvNotApplied,
    #[error("Can't read the version as a valid semver")]
    VersionError {
        source: node_semver::SemverError,
        version: String,
    },
}
