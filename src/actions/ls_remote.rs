use crate::app_config::AppConfig;
use crate::ni_remote;
use crate::user_version::UserVersion;

use colored::Colorize;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct ShowRemote {
    /// Filter versions by a user-defined version or a semver range
    #[arg(long)]
    filter: Option<UserVersion>,

    /// Show only LTS versions (optionally filter by LTS codename)  
    #[arg(long)]
    #[allow(clippy::option_option)]
    lts: Option<Option<String>>,

    /// Version sorting order
    #[arg(long, default_value = "asc")]
    sort: SortingMethod,

    /// Only show the latest matching version
    #[arg(long)]
    latest: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum SortingMethod {
    #[clap(name = "desc")]
    /// Sort versions in descending order (latest to earliest)
    Descending,
    #[clap(name = "asc")]
    /// Sort versions in ascending order (earliest to latest)
    Ascending,
}

/// Drain all elements but the last one
fn truncate_except_latest<T>(list: &mut Vec<T>) {
    let len = list.len();
    if len > 1 {
        list.swap(0, len - 1);
        list.truncate(1);
    }
}

impl super::command::Command for ShowRemote {
    type Error = Error;

    fn apply(self, config: &AppConfig) -> Result<(), Self::Error> {
        let mut all_versions = ni_remote::list(&config.node_dist_mirror)?;

        if let Some(lts) = &self.lts {
            match lts {
                Some(codename) => all_versions.retain(|v| {
                    v.lts
                        .as_ref()
                        .is_some_and(|v_lts| v_lts.eq_ignore_ascii_case(codename))
                }),
                None => all_versions.retain(|v| v.lts.is_some()),
            };
        }

        if let Some(filter) = &self.filter {
            all_versions.retain(|v| filter.matches(&v.version, config));
        }

        if self.latest {
            truncate_except_latest(&mut all_versions);
        }

        if let SortingMethod::Descending = self.sort {
            all_versions.reverse();
        }

        if all_versions.is_empty() {
            eprintln!("{}", "No versions were found!".red());
            return Ok(());
        }

        for version in &all_versions {
            print!("{}", version.version);
            if let Some(lts) = &version.lts {
                print!("{}", format!(" ({lts})").cyan());
            }
            println!();
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    HttpError {
        #[from]
        source: crate::http::Error,
    },
}
