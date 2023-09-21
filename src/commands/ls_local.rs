use crate::vsetter::{list_aliases, StoredVSetter};
use crate::config::EnmConfig;
use crate::version_now::version_now;
use crate::version::Version;
use colored::Colorize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(clap::Parser, Debug)]
pub struct LsLocal {}

impl super::command::Command for LsLocal {
    type Error = Error;

    fn apply(self, config: &EnmConfig) -> Result<(), Self::Error> {
        let base_dir = config.installations_dir();
        let mut versions = crate::version_installed::list(base_dir)
            .map_err(|source| Error::CantListLocallyInstalledVersion { source })?;
        versions.insert(0, Version::Bypassed);
        versions.sort();
        let vsetteres_hash =
            generate_aliases_hash(config).map_err(|source| Error::CantReadVSetteres { source })?;
        let curr_version = version_now(config).ok().flatten();

        for version in versions {
            let version_aliases = match vsetteres_hash.get(&version.v_str()) {
                None => String::new(),
                Some(versions) => {
                    let version_string = versions
                        .iter()
                        .map(StoredVSetter::name)
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(" {}", version_string.dimmed())
                }
            };

            let version_str = format!("* {version}{version_aliases}");

            if curr_version == Some(version) {
                println!("{}", version_str.cyan());
            } else {
                println!("{version_str}");
            }
        }
        Ok(())
    }
}

fn generate_aliases_hash(config: &EnmConfig) -> std::io::Result<HashMap<String, Vec<StoredVSetter>>> {
    let mut vsetteres = list_aliases(config)?;
    let mut hashmap: HashMap<String, Vec<StoredVSetter>> = HashMap::with_capacity(vsetteres.len());
    for vsetter in vsetteres.drain(..) {
        if let Some(value) = hashmap.get_mut(vsetter.s_ver()) {
            value.push(vsetter);
        } else {
            hashmap.insert(vsetter.s_ver().into(), vec![vsetter]);
        }
    }
    Ok(hashmap)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't list locally installed versions: {}", source)]
    CantListLocallyInstalledVersion {
        source: crate::version_installed::Error,
    },
    #[error("Can't read vsetteres: {}", source)]
    CantReadVSetteres { source: std::io::Error },
}
