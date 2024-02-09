use crate::available_versions;
use crate::app_config::AppConfig;
use crate::symlinked;
use crate::machine_semver;
use crate::user_version::UserVersion;
use crate::version::Version;
use colored::Colorize;
use log::info;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug)]
pub struct ApplicableVersion {
    path: PathBuf,
    version: Version,
}

impl ApplicableVersion {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn version(&self) -> &Version {
        &self.version
    }
}

pub fn user_version_in<'a>(
    requested_version: &'a UserVersion,
    config: &'a AppConfig,
) -> Result<Option<ApplicableVersion>, Error> {
    let all_versions = available_versions::list(config.installations_dir())
        .map_err(|source| Error::VersionListing { source })?;

    let result = if let UserVersion::Full(Version::Bypassed) = requested_version {
        info!(
            "Skipping ENM: using {} node",
            machine_semver::display_name().cyan()
        );
        Some(ApplicableVersion {
            path: machine_semver::path(),
            version: Version::Bypassed,
        })
    } else if let Some(alias_name) = requested_version.alias_name() {
        let alias_path = config.aliases_dir().join(&alias_name);
        let system_path = machine_semver::path();
        if matches!(symlinked::shallow_read_symlink(&alias_path), Ok(shallow_path) if shallow_path == system_path)
        {
            info!(
                "Skipping ENM: using {} node",
                machine_semver::display_name().cyan()
            );
            Some(ApplicableVersion {
                path: alias_path,
                version: Version::Bypassed,
            })
        } else if alias_path.exists() {
            info!("Using Node for alias {}", alias_name.cyan());
            Some(ApplicableVersion {
                path: alias_path,
                version: Version::Alias(alias_name),
            })
        } else {
            return Err(Error::CantFindVersion {
                requested_version: requested_version.clone(),
            });
        }
    } else {
        let version_now = requested_version.to_version(&all_versions, config);
        version_now.map(|version| {
            info!("Using Node {}", version.to_string().cyan());
            let path = config
                .installations_dir()
                .join(version.to_string())
                .join("installation");

            ApplicableVersion {
                path,
                version: version.clone(),
            }
        })
    };

    Ok(result)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to find requested version: {}", requested_version)]
    CantFindVersion { requested_version: UserVersion },
    #[error("Unable to list local installed versions: {}", source)]
    VersionListing { source: available_versions::Error },
}
