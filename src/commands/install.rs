use super::command::Command;
use crate::vsetter::create_vsetter;
use crate::arch::get_safe_arch;
use crate::config::EnmConfig;
use crate::content_fetcher::{install_node_dist, Error as DownloaderError};
use crate::lts::LtsType;
use crate::outln;
use crate::progress::ProgressConfig;
use crate::enm_rnode_index;
use crate::user_version::UserVersion;
use crate::version::Version;
use crate::version_files::get_user_version_for_directory;
use colored::Colorize;
use log::debug;
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct Install {
    /// A version string. Can be a partial semver or a LTS version name by the format lts/NAME
    pub version: Option<UserVersion>,

    /// Install latest LTS
    #[clap(long, conflicts_with_all = &["version", "latest"])]
    pub lts: bool,

    /// Install latest version
    #[clap(long, conflicts_with_all = &["version", "lts"])]
    pub latest: bool,

    /// Show an interactive progress bar for the download
    /// status.
    #[clap(long, default_value_t)]
    #[arg(value_enum)]
    pub progress: ProgressConfig,
}

impl Install {
    fn version(self) -> Result<Option<UserVersion>, Error> {
        match self {
            Self {
                version: v,
                lts: false,
                latest: false,
                ..
            } => Ok(v),
            Self {
                version: None,
                lts: true,
                latest: false,
                ..
            } => Ok(Some(UserVersion::Full(Version::Lts(LtsType::Latest)))),
            Self {
                version: None,
                lts: false,
                latest: true,
                ..
            } => Ok(Some(UserVersion::Full(Version::Latest))),
            _ => Err(Error::TooManyVersionsProvided),
        }
    }
}

impl Command for Install {
    type Error = Error;

    fn apply(self, config: &EnmConfig) -> Result<(), Self::Error> {
        let current_dir = std::env::current_dir().unwrap();
        let show_progress = self.progress.enabled(config);

        let version_now = self
            .version()?
            .or_else(|| get_user_version_for_directory(current_dir, config))
            .ok_or(Error::CantInferVersion)?;

        let version = match version_now.clone() {
            UserVersion::Full(Version::Semver(actual_version)) => Version::Semver(actual_version),
            UserVersion::Full(v @ (Version::Bypassed | Version::VSetter(_))) => {
                return Err(Error::UninstallableVersion { version: v });
            }
            UserVersion::Full(Version::Lts(lts_type)) => {
                let available_versions: Vec<_> = enm_rnode_index::list(&config.node_dist_mirror)
                    .map_err(|source| Error::CantListRemoteVersions { source })?;
                let picked_version = lts_type
                    .pick_latest(&available_versions)
                    .ok_or_else(|| Error::CantFindRelevantLts {
                        lts_type: lts_type.clone(),
                    })?
                    .version
                    .clone();
                debug!(
                    "Resolved {} into Node version {}",
                    Version::Lts(lts_type).v_str().cyan(),
                    picked_version.v_str().cyan()
                );
                picked_version
            }
            UserVersion::Full(Version::Latest) => {
                let available_versions: Vec<_> = enm_rnode_index::list(&config.node_dist_mirror)
                    .map_err(|source| Error::CantListRemoteVersions { source })?;
                let picked_version = available_versions
                    .last()
                    .ok_or(Error::CantFindLatest)?
                    .version
                    .clone();
                debug!(
                    "Resolved {} into Node version {}",
                    Version::Latest.v_str().cyan(),
                    picked_version.v_str().cyan()
                );
                picked_version
            }
            version_now => {
                let available_versions: Vec<_> = enm_rnode_index::list(&config.node_dist_mirror)
                    .map_err(|source| Error::CantListRemoteVersions { source })?
                    .drain(..)
                    .map(|x| x.version)
                    .collect();

                version_now
                    .to_version(&available_versions, config)
                    .ok_or(Error::CantFindNodeVersion {
                        requested_version: version_now,
                    })?
                    .clone()
            }
        };

        // Automatically swap Apple Silicon to x64 arch for appropriate versions.
        let safe_arch = get_safe_arch(config.arch, &version);

        let version_str = format!("Node {}", &version);
        outln!(
            config,
            Info,
            "Installing {} ({})",
            version_str.cyan(),
            safe_arch.as_str()
        );

        match install_node_dist(
            &version,
            &config.node_dist_mirror,
            config.installations_dir(),
            safe_arch,
            show_progress,
        ) {
            Err(err @ DownloaderError::VersionAlreadyInstalled { .. }) => {
                outln!(config, Error, "{} {}", "warning:".bold().yellow(), err);
            }
            Err(source) => Err(Error::DownloadError { source })?,
            Ok(()) => {}
        };

        if !config.default_version_dir().exists() {
            debug!("Tagging {} as the default version", version.v_str().cyan());
            create_vsetter(config, "default", &version)?;
        }

        if let Some(tagged_vsetter) = version_now.inferred_alias() {
            tag_alias(config, &version, &tagged_vsetter)?;
        }

        // if config.corepack_enabled() {
        //     outln!(config, Info, "Enabling corepack for {}", version_str.cyan());
        //     enable_corepack(&version, config)?;
        // }

        Ok(())
    }
}

fn tag_alias(config: &EnmConfig, matched_version: &Version, vsetter: &Version) -> Result<(), Error> {
    let vsetter_name = vsetter.v_str();
    debug!(
        "Tagging {} as vsetter for {}",
        vsetter_name.cyan(),
        matched_version.v_str().cyan()
    );
    create_vsetter(config, &vsetter_name, matched_version)?;

    Ok(())
}

// fn enable_corepack(version: &Version, config: &EnmConfig) -> Result<(), Error> {
//     let corepack_path = version.installation_path(config);
//     let corepack_path = if cfg!(windows) {
//         corepack_path.join("corepack.cmd")
//     } else {
//         corepack_path.join("bin").join("corepack")
//     };
//     super::exec::Exec::new_for_version(version, corepack_path.to_str().unwrap(), &["enable"])
//         .apply(config)
//         .map_err(|source| Error::CorepackError { source })?;
//     Ok(())
// }
//
#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't download the requested binary: {}", source)]
    DownloadError { source: DownloaderError },
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error("Can't enable corepack: {source}")]
    CorepackError {
        #[from]
        source: super::exec::Error,
    },
    #[error("Can't find version in dotfiles. Please provide a version manually to the command.")]
    CantInferVersion,
    #[error("Having a hard time listing the remote versions: {}", source)]
    CantListRemoteVersions { source: crate::http::Error },
    #[error(
        "Can't find a Node version that matches {} in remote",
        requested_version
    )]
    CantFindNodeVersion { requested_version: UserVersion },
    #[error("Can't find relevant LTS named {}", lts_type)]
    CantFindRelevantLts { lts_type: crate::lts::LtsType },
    #[error("Can't find any versions in the upstream version index.")]
    CantFindLatest,
    #[error("The requested version is not installable: {}", version.v_str())]
    UninstallableVersion { version: Version },
    #[error("Too many versions provided. Please don't use --lts with a version string.")]
    TooManyVersionsProvided,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    #[test]
    fn test_set_default_on_new_installation() {
        let base_dir = tempfile::tempdir().unwrap();
        let config = EnmConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()));
        assert!(!config.default_version_dir().exists());

        Install {
            version: UserVersion::from_str("12.0.0").ok(),
            lts: false,
            latest: false,
            progress: ProgressConfig::Never,
        }
        .apply(&config)
        .expect("Can't install");

        assert!(config.default_version_dir().exists());
        assert_eq!(
            config.default_version_dir().canonicalize().ok(),
            config
                .installations_dir()
                .join("v12.0.0")
                .join("installation")
                .canonicalize()
                .ok()
        );
    }

    #[test]
    fn test_install_latest() {
        let base_dir = tempfile::tempdir().unwrap();
        let config = EnmConfig::default().with_base_dir(Some(base_dir.path().to_path_buf()));

        Install {
            version: None,
            lts: false,
            latest: true,
            progress: ProgressConfig::Never,
        }
        .apply(&config)
        .expect("Can't install");

        let available_versions: Vec<_> =
            enm_rnode_index::list(&config.node_dist_mirror).expect("Can't get node version list");
        let latest_version = available_versions.last().unwrap().version.clone();

        assert!(config.installations_dir().exists());
        assert!(config
            .installations_dir()
            .join(latest_version.to_string())
            .join("installation")
            .canonicalize()
            .unwrap()
            .exists());
    }
}
