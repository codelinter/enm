use thiserror::Error;

use crate::app_config::AppConfig;
use crate::machine_semver;
use crate::version::Version;

pub fn version_now(config: &AppConfig) -> Result<Option<Version>, Error> {
    let plural_ctx = config.plural_ctx().ok_or(Error::EnvNotApplied)?;

    if plural_ctx.read_link().ok() == Some(machine_semver::path()) {
        return Ok(Some(Version::Bypassed));
    }

    if let Ok(resolved_path) = std::fs::canonicalize(plural_ctx) {
        let installation_path = resolved_path
            .parent()
            .expect("plural_ctx path can't be in the root");
        let file_name = installation_path
            .file_name()
            .expect("Unable to get filename")
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
    #[error("`enm source` was not applied in this context.\nUnable to find enm's environment variables")]
    EnvNotApplied,
    #[error("Unable to read the version as a valid semver")]
    VersionError {
        source: node_semver::SemverError,
        version: String,
    },
}
