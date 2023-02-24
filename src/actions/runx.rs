use super::command::Command as Cmd;
use crate::app_config::AppConfig;
use crate::outln;
use crate::reader_uv::ReaderUV;
use crate::user_version::UserVersion;
use crate::user_version_in::{user_version_in, Error as UserInputError};
use colored::Colorize;
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Debug, clap::Parser)]
#[clap(trailing_var_arg = true)]
pub struct RunX {
    /// Either an explicit version, or a filename with the version written in it
    #[clap(long = "with")]
    version: Option<ReaderUV>,
    /// Deprecated. This is the default now.
    #[clap(long = "with-file", hide = true)]
    with_file: bool,
    /// The command to run
    arguments: Vec<String>,
}

impl RunX {
    pub(crate) fn new_for_version(
        version: &crate::version::Version,
        cmd: &str,
        arguments: &[&str],
    ) -> Self {
        let reader = ReaderUV::Direct(UserVersion::Full(version.clone()));
        let args: Vec<_> = std::iter::once(cmd)
            .chain(arguments.iter().copied())
            .map(String::from)
            .collect();
        Self {
            version: Some(reader),
            with_file: false,
            arguments: args,
        }
    }
}

impl Cmd for RunX {
    type Error = Error;

    fn apply(self, config: &AppConfig) -> Result<(), Self::Error> {
        if self.with_file {
            outln!(
                config,
                Error,
                "{} {} is deprecated. This is now the default.",
                "warning:".yellow().bold(),
                "--with-file".italic()
            );
        }

        let (binary, arguments) = self
            .arguments
            .split_first()
            .ok_or(Error::NoBinaryProvided)?;

        let version = self
            .version
            .unwrap_or_else(|| {
                let current_dir = std::env::current_dir().unwrap();
                ReaderUV::Path(current_dir)
            })
            .into_user_version(config)
            .ok_or(Error::CantConjectVersion)?;

        let applicable_version = user_version_in(&version, config)
            .map_err(|source| Error::ApplicableVersionError { source })?
            .ok_or(Error::VersionNotFound { version })?;

        #[cfg(windows)]
        let bin_path = applicable_version.path().to_path_buf();

        #[cfg(unix)]
        let bin_path = applicable_version.path().join("bin");

        let path_env = {
            let paths_env = std::env::var_os("PATH").ok_or(Error::CantReadPathVariable)?;
            let mut paths: Vec<_> = std::env::split_paths(&paths_env).collect();
            paths.insert(0, bin_path);
            std::env::join_paths(paths)
                .map_err(|source| Error::CantAddPathToEnvironment { source })?
        };

        log::debug!("Running {} with PATH={:?}", binary, path_env);

        let exit_status = Command::new(binary)
            .args(arguments)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env("PATH", path_env)
            .spawn()
            .map_err(|source| Error::CantSpawnProgram {
                source,
                binary: binary.to_string(),
            })?
            .wait()
            .expect("Failed to grab exit code");

        let code = exit_status.code().ok_or(Error::CantReadProcessExitCode)?;
        std::process::exit(code);
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to spawn program: {source}\nMaybe the program {} does not exist on not available in PATH?", binary.bold())]
    CantSpawnProgram {
        source: std::io::Error,
        binary: String,
    },
    #[error("Unable to read path environment variable")]
    CantReadPathVariable,
    #[error("Unable to add path to environment variable: {}", source)]
    CantAddPathToEnvironment { source: std::env::JoinPathsError },
    #[error("Unable to find version in dotfiles. Please provide a version manually to the command.")]
    CantConjectVersion,
    #[error("Requested version {} is not currently installed", version)]
    VersionNotFound { version: UserVersion },
    #[error(transparent)]
    ApplicableVersionError {
        #[from]
        source: UserInputError,
    },
    #[error("Unable to read exit code from process.\nMaybe the process was killed using a signal?")]
    CantReadProcessExitCode,
    #[error("command not provided. Please provide a command to run as an argument, like {} or {}.\n{} {}", "node".italic(), "bash".italic(), "example:".yellow().bold(), "enm run --with=20 node -v".italic().yellow())]
    NoBinaryProvided,
}
