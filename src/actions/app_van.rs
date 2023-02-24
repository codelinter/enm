use super::command::Command;
use crate::app_config::AppConfig;
use crate::symlinked::symlink_dir;
use crate::outln;
use crate::path_ext::PathExt;
use crate::terminators_entry::{conjectr_shell, Shell, Terms};
use clap::ValueEnum;
use colored::Colorize;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;

#[derive(clap::Parser, Debug, Default)]
pub struct AppVan {
    #[clap(long)]
    shell: Option<Terms>,
    #[clap(long, conflicts_with = "shell")]
    json: bool,
    #[clap(long, hide = true)]
    plural: bool,
    /// Trigger NodeJS version change on entering a node project folder
    #[clap(long)]
    on_enter: bool,
}

fn create_symlink_path() -> String {
    format!(
        "{}_{}",
        std::process::id(),
        chrono::Utc::now().timestamp_millis(),
    )
}

fn create_symlink(config: &AppConfig) -> Result<std::path::PathBuf, Error> {
    let base_dir = config.plural_ctx_storage().ensure_exists_silently();
    let mut temp_dir = base_dir.join(create_symlink_path());

    while temp_dir.exists() {
        temp_dir = base_dir.join(create_symlink_path());
    }

    match symlink_dir(config.version_std_dir(), &temp_dir) {
        Ok(()) => Ok(temp_dir),
        Err(source) => Err(Error::CantCreateSymlink { source, temp_dir }),
    }
}

impl Command for AppVan {
    type Error = Error;

    fn apply(self, config: &AppConfig) -> Result<(), Self::Error> {
        if self.plural {
            outln!(
                config,
                Error,
                "{} {} is deprecated. This is now the default.",
                "warning:".yellow().bold(),
                "--multi".italic()
            );
        }

        let plural_ctx = create_symlink(config)?;
        let base_dir = config.base_dir_with_default();

        let env_vars = [
            ("ENM_SHIM", plural_ctx.to_str().unwrap()),
            ("ENM_SIFT_METHOD", config.sift_method().as_str()),
            ("ENM_DIR", base_dir.to_str().unwrap()),
        ];

        if self.json {
            println!(
                "{}",
                serde_json::to_string(&HashMap::from(env_vars)).unwrap()
            );
            return Ok(());
        }

        let shell: Box<dyn Shell> = self
            .shell
            .map(Into::into)
            .or_else(conjectr_shell)
            .ok_or(Error::CantConjectShell)?;

        let binary_path = if cfg!(windows) {
            shell.path(&plural_ctx)
        } else {
            shell.path(&plural_ctx.join("bin"))
        };

        println!("{}", binary_path?);

        for (name, value) in &env_vars {
            println!("{}", shell.set_env_var(name, value));
        }

        if self.on_enter {
            println!("{}", shell.on_enter(config)?);
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "{}\n{}\n{}\n{}",
        "Unable to conjectr shell!",
        "enm can't conjectr your shell based on the process tree.",
        "Maybe it is unsupported? we support the following shells:",
        shells_as_string()
    )]
    CantConjectShell,
    #[error("Symlink problem at {temp_dir:?} for project at path {source}")]
    CantCreateSymlink {
        #[source]
        source: std::io::Error,
        temp_dir: std::path::PathBuf,
    },
    #[error(transparent)]
    ShellError {
        #[from]
        source: anyhow::Error,
    },
}

fn shells_as_string() -> String {
    Terms::value_variants()
        .iter()
        .map(|x| format!("* {x}"))
        .collect::<Vec<_>>()
        .join("\n")
}
