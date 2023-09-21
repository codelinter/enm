use crate::config::EnmConfig;
use crate::fs;
use crate::version_installed;
use crate::system_version;
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

pub fn user_opted_version<'a>(
    requested_version: &'a UserVersion,
    config: &'a EnmConfig,
) -> Result<Option<ApplicableVersion>, Error> {
    let all_versions = version_installed::list(config.installations_dir())
        .map_err(|source| Error::VersionListing { source })?;

    let result = if let UserVersion::Full(Version::Bypassed) = requested_version {
        info!(
            "Bypassing enm: using {} node",
            system_version::display_name().cyan()
        );
        Some(ApplicableVersion {
            path: system_version::path(),
            version: Version::Bypassed,
        })
    } else if let Some(vsetter_name) = requested_version.vsetter_name() {
        let vsetter_path = config.vsetteres_dir().join(&vsetter_name);
        let system_path = system_version::path();
        if matches!(fs::shallow_read_symlink(&vsetter_path), Ok(shallow_path) if shallow_path == system_path)
        {
            info!(
                "Bypassing enm: using {} node",
                system_version::display_name().cyan()
            );
            Some(ApplicableVersion {
                path: vsetter_path,
                version: Version::Bypassed,
            })
        } else if vsetter_path.exists() {
            info!("Using Node for vsetter {}", vsetter_name.cyan());
            Some(ApplicableVersion {
                path: vsetter_path,
                version: Version::VSetter(vsetter_name),
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
    #[error("Can't find requested version: {}", requested_version)]
    CantFindVersion { requested_version: UserVersion },
    #[error("Can't list local installed versions: {}", source)]
    VersionListing { source: version_installed::Error },
}
