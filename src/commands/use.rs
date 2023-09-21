use super::command::Command;
use super::install::Install;
use crate::version_now::version_now;
use crate::fs;
use crate::version_installed;
use crate::outln;
use crate::shell;
use crate::system_version;
use crate::user_version::UserVersion;
use crate::version::Version;
use crate::cd_file_changer::VersionFileStrategy;
use crate::{config::EnmConfig, version_uv_reader::UserVersionReader};
use colored::Colorize;
use std::path::Path;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Use {
    version: Option<UserVersionReader>,
    /// Install the version if it isn't installed yet
    #[clap(long)]
    install_if_missing: bool,

    /// Don't output a message identifying the version being used
    /// if it will not change due to execution of this command
    #[clap(long)]
    silent_if_unchanged: bool,
}

impl Command for Use {
    type Error = Error;

    fn apply(self, config: &EnmConfig) -> Result<(), Self::Error> {
        let runway_path = config.runway_path().ok_or(Error::EnmEnvWasNotSourced)?;
        warn_if_runway_path_not_in_path_env_var(runway_path, config);

        let all_versions = version_installed::list(config.installations_dir())
            .map_err(|source| Error::VersionListingError { source })?;
        let requested_version = self
            .version
            .unwrap_or_else(|| {
                let current_dir = std::env::current_dir().unwrap();
                UserVersionReader::Path(current_dir)
            })
            .into_user_version(config)
            .ok_or_else(|| match config.cd_file_changer() {
                VersionFileStrategy::Local => InferVersionError::Local,
                VersionFileStrategy::Recursive => InferVersionError::Recursive,
            })
            .map_err(|source| Error::CantInferVersion { source })?;

        let (message, version_path) = if let UserVersion::Full(Version::Bypassed) =
            requested_version
        {
            let message = format!(
                "Bypassing enm: using {} node",
                system_version::display_name().cyan()
            );
            (message, system_version::path())
        } else if let Some(vsetter_name) = requested_version.vsetter_name() {
            let vsetter_path = config.vsetteres_dir().join(&vsetter_name);
            let system_path = system_version::path();
            if matches!(fs::shallow_read_symlink(&vsetter_path), Ok(shallow_path) if shallow_path == system_path)
            {
                let message = format!(
                    "Bypassing enm: using {} node",
                    system_version::display_name().cyan()
                );
                (message, system_path)
            } else if vsetter_path.exists() {
                let message = format!("Using Node for vsetter {}", vsetter_name.cyan());
                (message, vsetter_path)
            } else {
                install_new_version(requested_version, config, self.install_if_missing)?;
                return Ok(());
            }
        } else {
            let version_now = requested_version.to_version(&all_versions, config);
            if let Some(version) = version_now {
                let version_path = config
                    .installations_dir()
                    .join(version.to_string())
                    .join("installation");
                let message = format!("Using Node {}", version.to_string().cyan());
                (message, version_path)
            } else {
                install_new_version(requested_version, config, self.install_if_missing)?;
                return Ok(());
            }
        };

        if !self.silent_if_unchanged || will_version_change(&version_path, config) {
            outln!(config, Info, "{}", message);
        }

        if let Some(runways_path) = runway_path.parent() {
            std::fs::create_dir_all(runways_path).map_err(|_err| {
                Error::MultishellDirectoryCreationIssue {
                    path: runways_path.to_path_buf(),
                }
            })?;
        }

        replace_symlink(&version_path, runway_path)
            .map_err(|source| Error::SymlinkingCreationIssue { source })?;

        Ok(())
    }
}

fn will_version_change(resolved_path: &Path, config: &EnmConfig) -> bool {
    let version_now_path = version_now(config)
        .unwrap_or(None)
        .map(|v| v.installation_path(config));

    version_now_path.as_deref() != Some(resolved_path)
}

fn install_new_version(
    requested_version: UserVersion,
    config: &EnmConfig,
    install_if_missing: bool,
) -> Result<(), Error> {
    if !install_if_missing && !should_install_interactively(&requested_version) {
        return Err(Error::CantFindVersion {
            version: requested_version,
        });
    }

    Install {
        version: Some(requested_version.clone()),
        ..Install::default()
    }
    .apply(config)
    .map_err(|source| Error::InstallError { source })?;

    Use {
        version: Some(UserVersionReader::Direct(requested_version)),
        install_if_missing: true,
        silent_if_unchanged: false,
    }
    .apply(config)?;

    Ok(())
}

/// Tries to delete `from`, and then tries to symlink `from` to `to` anyway.
/// If the symlinking fails, it will return the errors in the following order:
/// * The deletion error (if exists)
/// * The creation error
///
/// This way, we can create a symlink if it is missing.
fn replace_symlink(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    let symlink_deletion_result = fs::remove_symlink_dir(to);
    match fs::symlink_dir(from, to) {
        ok @ Ok(()) => ok,
        err @ Err(_) => symlink_deletion_result.and(err),
    }
}

fn should_install_interactively(requested_version: &UserVersion) -> bool {
    use std::io::{IsTerminal, Write};

    if !(std::io::stdout().is_terminal() && std::io::stdin().is_terminal()) {
        return false;
    }

    let error_message = format!(
        "Can't find an installed Node version matching {}.",
        requested_version.to_string().italic()
    );
    eprintln!("{}", error_message.red());
    let do_you_want = format!("Do you want to install it? {} [y/N]:", "answer".bold());
    eprint!("{} ", do_you_want.yellow());
    std::io::stdout().flush().unwrap();
    let mut s = String::new();
    std::io::stdin()
        .read_line(&mut s)
        .expect("Can't read user input");

    s.trim().to_lowercase() == "y"
}

fn warn_if_runway_path_not_in_path_env_var(
    runway_path: &std::path::Path,
    config: &EnmConfig,
) {
    if let Some(path_var) = std::env::var_os("PATH") {
        let bin_path = if cfg!(unix) {
            runway_path.join("bin")
        } else {
            runway_path.to_path_buf()
        };

        let fixed_path = bin_path.to_str().and_then(shell::maybe_fix_windows_path);
        let fixed_path = fixed_path.as_deref();

        for path in std::env::split_paths(&path_var) {
            if bin_path == path || fixed_path == path.to_str() {
                return;
            }
        }
    }

    outln!(
        config, Error,
        "{} {}\n{}",
        "warning:".yellow().bold(),
        "The current Node.js path is not on your PATH environment variable.".yellow(),
        "You should setup your shell profile to evaluate `enm env`".yellow()
    );
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't create the symlink: {}", source)]
    SymlinkingCreationIssue { source: std::io::Error },
    #[error(transparent)]
    InstallError { source: <Install as Command>::Error },
    #[error("Can't get locally installed versions: {}", source)]
    VersionListingError { source: version_installed::Error },
    #[error("Requested version {} is not currently installed", version)]
    CantFindVersion { version: UserVersion },
    #[error(transparent)]
    CantInferVersion {
        #[from]
        source: InferVersionError,
    },
    #[error(
        "{}\n{}",
        "We can't find the necessary environment variables to replace the Node version.",
        "You should setup your shell profile to evaluate `enm env`"
    )]
    EnmEnvWasNotSourced,
    #[error("Can't create the multishell directory: {}", path.display())]
    MultishellDirectoryCreationIssue { path: std::path::PathBuf },
}

#[derive(Debug, Error)]
pub enum InferVersionError {
    #[error("Can't find version in dotfiles. Please provide a version manually to the command.")]
    Local,
    #[error("Could not find any version to use. Maybe you don't have a default version set?\nTry running `enm default <VERSION>` to set one,\nor create a .node-version file inside your project to declare a Node.js version.")]
    Recursive,
}
