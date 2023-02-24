use super::command::Command;
use super::firstrun::FirstRun;
use crate::available_versions;
use crate::symlinked;
use crate::outln;
use crate::terminators_entry;
use crate::sift_method::SiftMethod;
use crate::machine_semver;
use crate::user_version::UserVersion;
use crate::version::Version;
use crate::version_now::version_now;
use crate::{app_config::AppConfig, reader_uv::ReaderUV};
use colored::Colorize;
use std::path::Path;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct Switch {
    version: Option<ReaderUV>,
    /// Install the version if it isn't installed yet
    #[clap(long)]
    install_if_missing: bool,

    #[clap(long)]
    caps_lock_when_needed: bool,
}

impl Command for Switch {
    type Error = Error;

    fn apply(self, config: &AppConfig) -> Result<(), Self::Error> {
        let plural_ctx = config.plural_ctx().ok_or(Error::EnmEnvWasNotSourced)?;
        warn_if_plural_ctx_not_in_path_env_var(plural_ctx, config);

        let all_versions = available_versions::list(config.installations_dir())
            .map_err(|source| Error::VersionListingError { source })?;
        let requested_version = self
            .version
            .unwrap_or_else(|| {
                let current_dir = std::env::current_dir().unwrap();
                ReaderUV::Path(current_dir)
            })
            .into_user_version(config)
            .ok_or_else(|| match config.sift_method() {
                SiftMethod::Local => ConjectVersionError::Local,
                SiftMethod::Recursive => ConjectVersionError::Recursive,
            })
            .map_err(|source| Error::CantConjectVersion { source })?;

        let (message, version_path) = if let UserVersion::Full(Version::Bypassed) =
            requested_version
        {
            let message = format!(
                "Skipping ENM: using {} node",
                machine_semver::display_name().cyan()
            );
            (message, machine_semver::path())
        } else if let Some(alias_name) = requested_version.alias_name() {
            let alias_path = config.aliases_dir().join(&alias_name);
            let system_path = machine_semver::path();
            if matches!(symlinked::shallow_read_symlink(&alias_path), Ok(shallow_path) if shallow_path == system_path)
            {
                let message = format!(
                    "Skipping ENM: using {} node",
                    machine_semver::display_name().cyan()
                );
                (message, system_path)
            } else if alias_path.exists() {
                let message = format!("Using Node for alias {}", alias_name.cyan());
                (message, alias_path)
            } else {
                first_run_new_version(requested_version, config, self.install_if_missing)?;
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
                first_run_new_version(requested_version, config, self.install_if_missing)?;
                return Ok(());
            }
        };

        if !self.caps_lock_when_needed || will_version_change(&version_path, config) {
            outln!(config, Info, "{}", message);
        }

        if let Some(shim_path) = plural_ctx.parent() {
            std::fs::create_dir_all(shim_path).map_err(|_err| {
                Error::PluralCtxDirectoryCreationIssue {
                    path: shim_path.to_path_buf(),
                }
            })?;
        }

        replace_symlink(&version_path, plural_ctx)
            .map_err(|source| Error::SymlinkingCreationIssue { source })?;

        Ok(())
    }
}

fn will_version_change(resolved_path: &Path, config: &AppConfig) -> bool {
    let version_now_path = version_now(config)
        .unwrap_or(None)
        .map(|v| v.installation_path(config));

    version_now_path.as_deref() != Some(resolved_path)
}

fn first_run_new_version(
    requested_version: UserVersion,
    config: &AppConfig,
    install_if_missing: bool,
) -> Result<(), Error> {
    if !install_if_missing && !should_install_interactively(&requested_version) {
        return Err(Error::CantFindVersion {
            version: requested_version,
        });
    }

    FirstRun {
        version: Some(requested_version.clone()),
        ..FirstRun::default()
    }
    .apply(config)
    .map_err(|source| Error::InstallError { source })?;

    Switch {
        version: Some(ReaderUV::Direct(requested_version)),
        install_if_missing: true,
        caps_lock_when_needed: false,
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
    let symlink_deletion_result = symlinked::remove_symlink_dir(to);
    match symlinked::symlink_dir(from, to) {
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
        "NodeJS with version {} not found in your system",
        requested_version.to_string().italic()
    );
    eprintln!("{}", error_message.red());
    let do_you_want = format!("Type y to install {} [y/N]:", "answer".bold());
    eprint!("{} ", do_you_want.cyan());
    std::io::stdout().flush().unwrap();
    let mut s = String::new();
    std::io::stdin()
        .read_line(&mut s)
        .expect("Unable to read user input");

    s.trim().to_lowercase() == "y"
}

fn warn_if_plural_ctx_not_in_path_env_var(
    plural_ctx: &std::path::Path,
    config: &AppConfig,
) {
    if let Some(path_var) = std::env::var_os("PATH") {
        let bin_path = if cfg!(unix) {
            plural_ctx.join("bin")
        } else {
            plural_ctx.to_path_buf()
        };

        let fixed_path = bin_path.to_str().and_then(terminators_entry::microsoft_prod_patch_path);
        let fixed_path = fixed_path.as_deref();

        for path in std::env::split_paths(&path_var) {
            if bin_path == path || fixed_path == path.to_str() {
                return;
            }
        }
    }

    outln!(
        config, Error,
        "{} {}\n",
        "Woh:".yellow().bold(),
        "Did you forget to eval `enm source`?".yellow()
    );
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to create the symlink: {}", source)]
    SymlinkingCreationIssue { source: std::io::Error },
    #[error(transparent)]
    InstallError { source: <FirstRun as Command>::Error },
    #[error("Unable to fetch locally installed versions: {}", source)]
    VersionListingError { source: available_versions::Error },
    #[error("Requested NodeJS version {} is not installed in your system", version)]
    CantFindVersion { version: UserVersion },
    #[error(transparent)]
    CantConjectVersion {
        #[from]
        source: ConjectVersionError,
    },
    #[error(
        "{}",
        "Did you forget to eval `enm source`?",
    )]
    EnmEnvWasNotSourced,
    #[error("Unable to create the plural_ctx directory: {}", path.display())]
    PluralCtxDirectoryCreationIssue { path: std::path::PathBuf },
}

#[derive(Debug, Error)]
pub enum ConjectVersionError {
    #[error("Unable to detect NodeJS version using dotfiles. Please manually provide the version insteads.")]
    Local,
    #[error("Could not find any version to switch. You may not a default version set yet\nTry running `enm default <VERSION>` to set one,\nor create a .node-version or .nvmrc file inside your project to declare a Node.js version.")]
    Recursive,
}
