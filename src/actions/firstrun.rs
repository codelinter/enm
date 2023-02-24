use super::command::Command;
use crate::alias::create_alias;
use crate::app_config::AppConfig;
use crate::cpu_arch::get_safe_cpu_arch;
use crate::fetcher::{install_node_dist, Error as DownloaderError};
use crate::loaders::ProgressConfig;
use crate::long_term_usage::LongTermType;
use crate::ni_remote;
use crate::outln;
use crate::user_version::UserVersion;
use crate::version::Version;
use crate::version_files::get_user_version_for_directory;
use colored::Colorize;
use log::debug;
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct FirstRun {
    /// A version string. Can be a partial semver or a LTS version name by the format lts/NAME
    pub version: Option<UserVersion>,

    /// Install latest LTS
    #[clap(long, conflicts_with_all = &["version", "latest"])]
    pub lts: bool,

    /// Install latest version
    #[clap(long, conflicts_with_all = &["version", "lts"])]
    pub latest: bool,

    /// Show an interactive loaders bar for the download
    /// status.
    #[clap(long, default_value_t)]
    #[arg(value_enum)]
    pub loaders: ProgressConfig,
}

impl FirstRun {
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
            } => Ok(Some(UserVersion::Full(Version::Lts(LongTermType::Latest)))),
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

impl Command for FirstRun {
    type Error = Error;

    fn apply(self, config: &AppConfig) -> Result<(), Self::Error> {
        let current_dir = std::env::current_dir().unwrap();
        let show_loaders = self.loaders.enabled(config);

        let version_now = self
            .version()?
            .or_else(|| get_user_version_for_directory(current_dir, config))
            .ok_or(Error::CantConjectVersion)?;

        let version = match version_now.clone() {
            UserVersion::Full(Version::Semver(actual_version)) => Version::Semver(actual_version),
            UserVersion::Full(v @ (Version::Bypassed | Version::Alias(_))) => {
                return Err(Error::UninstallableVersion { version: v });
            }
            UserVersion::Full(Version::Lts(lts_type)) => {
                let available_versions: Vec<_> = ni_remote::list(&config.node_dist_mirror)
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
                let available_versions: Vec<_> = ni_remote::list(&config.node_dist_mirror)
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
                let available_versions: Vec<_> = ni_remote::list(&config.node_dist_mirror)
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

        // Automatically swap Apple Silicon to x64 cpu_arch for appropriate versions.
        let safe_cpu_arch = get_safe_cpu_arch(config.cpu_arch, &version);

        let version_str = format!("Node {}", &version);
        outln!(
            config,
            Info,
            "Installing {} ({})",
            version_str.cyan(),
            safe_cpu_arch.as_str()
        );

        match install_node_dist(
            &version,
            &config.node_dist_mirror,
            config.installations_dir(),
            safe_cpu_arch,
            show_loaders,
        ) {
            Err(err @ DownloaderError::VersionAlreadyInstalled { .. }) => {
                outln!(config, Error, "{} {}", "warning:".bold().yellow(), err);
            }
            Err(source) => Err(Error::DownloadError { source })?,
            Ok(()) => {}
        };

        if !config.version_std_dir().exists() {
            debug!("Tagging {} as the default version", version.v_str().cyan());
            create_alias(config, "default", &version)?;
        }

        if let Some(tagged_alias) = version_now.conjectrred_alias() {
            tag_alias(config, &version, &tagged_alias)?;
        }

        if config.corepack_enabled() {
            outln!(config, Info, "Enabling corepack for {}", version_str.cyan());
            enable_corepack(&version, config)?;
        }

        Ok(())
    }
}

fn tag_alias(config: &AppConfig, matched_version: &Version, alias: &Version) -> Result<(), Error> {
    let alias_name = alias.v_str();
    debug!(
        "Tagging {} as alias for {}",
        alias_name.cyan(),
        matched_version.v_str().cyan()
    );
    create_alias(config, &alias_name, matched_version)?;

    Ok(())
}

fn enable_corepack(version: &Version, config: &AppConfig) -> Result<(), Error> {
    let corepack_path = version.installation_path(config);
    let corepack_path = if cfg!(windows) {
        corepack_path.join("corepack.cmd")
    } else {
        corepack_path.join("bin").join("corepack")
    };
    super::runx::RunX::new_for_version(version, corepack_path.to_str().unwrap(), &["enable"])
        .apply(config)
        .map_err(|source| Error::CorepackError { source })?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to download the requested binary: {}", source)]
    DownloadError { source: DownloaderError },
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error("Unable to enable corepack: {source}")]
    CorepackError {
        #[from]
        source: super::runx::Error,
    },
    #[error("Unable to find version in dotfiles. Please provide a version manually to the command.")]
    CantConjectVersion,
    #[error("Having a hard time listing the remote versions: {}", source)]
    CantListRemoteVersions { source: crate::http::Error },
    #[error(
        "Unable to find a Node version that matches {} in remote",
        requested_version
    )]
    CantFindNodeVersion { requested_version: UserVersion },
    #[error("Unable to find relevant LTS named {}", lts_type)]
    CantFindRelevantLts { lts_type: crate::long_term_usage::LongTermType },
    #[error("Unable to find any versions in the upstream version index.")]
    CantFindLatest,
    #[error("The requested version is not installable: {}", version.v_str())]
    UninstallableVersion { version: Version },
    #[error("Too many versions provided. Please don't use --lts with a version string.")]
    TooManyVersionsProvided,
}
