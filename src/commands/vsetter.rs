use super::command::Command;
use crate::vsetter::create_vsetter;
use crate::user_opted_version::{
    user_opted_version, Error as ApplicableVersionError,
};
use crate::config::EnmConfig;
use crate::user_version::UserVersion;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct VSetter {
    pub(crate) to_version: UserVersion,
    pub(crate) name: String,
}

impl Command for VSetter {
    type Error = Error;

    fn apply(self, config: &EnmConfig) -> Result<(), Self::Error> {
        let applicable_version = user_opted_version(&self.to_version, config)
            .map_err(|source| Error::CantUnderstandVersion { source })?
            .ok_or(Error::VersionNotFound {
                version: self.to_version,
            })?;

        create_vsetter(config, &self.name, applicable_version.version())
            .map_err(|source| Error::CantCreateSymlink { source })?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't create symlink for vsetter: {}", source)]
    CantCreateSymlink { source: std::io::Error },
    #[error("Version {} not found locally", version)]
    VersionNotFound { version: UserVersion },
    #[error(transparent)]
    CantUnderstandVersion { source: ApplicableVersionError },
}
