use super::command::Command;
use crate::alias::create_alias;
use crate::app_config::AppConfig;
use crate::user_version::UserVersion;
use crate::user_version_in::{user_version_in, Error as ApplicableVersionError};
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Alias {
    pub(crate) to_version: UserVersion,
    pub(crate) name: String,
}

impl Command for Alias {
    type Error = Error;

    fn apply(self, config: &AppConfig) -> Result<(), Self::Error> {
        let applicable_version = user_version_in(&self.to_version, config)
            .map_err(|source| Error::CantUnderstandVersion { source })?
            .ok_or(Error::VersionNotFound {
                version: self.to_version,
            })?;

        create_alias(config, &self.name, applicable_version.version())
            .map_err(|source| Error::CantCreateSymlink { source })?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to create symlink for alias: {}", source)]
    CantCreateSymlink { source: std::io::Error },
    #[error("Version {} not found locally", version)]
    VersionNotFound { version: UserVersion },
    #[error(transparent)]
    CantUnderstandVersion { source: ApplicableVersionError },
}
