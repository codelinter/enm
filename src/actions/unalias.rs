use super::command::Command;
use crate::symlinked::remove_symlink_dir;
use crate::user_version::UserVersion;
use crate::version::Version;
use crate::{app_config::AppConfig, user_version_in};
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Unalias {
    pub(crate) requested_alias: String,
}

impl Command for Unalias {
    type Error = Error;

    fn apply(self, config: &AppConfig) -> Result<(), Self::Error> {
        let requested_version = user_version_in::user_version_in(
            &UserVersion::Full(Version::Alias(self.requested_alias.clone())),
            config,
        )
        .ok()
        .flatten()
        .ok_or(Error::AliasNotFound {
            requested_alias: self.requested_alias,
        })?;

        remove_symlink_dir(requested_version.path())
            .map_err(|source| Error::CantDeleteSymlink { source })?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to delete symlink: {}", source)]
    CantDeleteSymlink { source: std::io::Error },
    #[error("Requested alias {} not found", requested_alias)]
    AliasNotFound { requested_alias: String },
}
