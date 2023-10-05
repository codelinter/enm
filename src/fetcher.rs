use crate::cpu_arch::CPUArch;
use crate::loaders::ResponseProgress;
use crate::pressed::{Archive, Error as ExtractError};
use crate::prtl_folder::DirectoryPortal;
use crate::version::Version;
use indicatif::ProgressDrawTarget;
use log::debug;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    HttpError {
        #[from]
        source: crate::http::Error,
    },
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error("Unable to extract the file: {}", source)]
    CantExtractFile {
        #[from]
        source: ExtractError,
    },
    #[error("The fetched archive seems empty")]
    EmptyTape,
    #[error("{} for {} not found upstream.\nYou can `enm show-remote` to see available versions or try a different `--cpu_arch`.", version, cpu_arch)]
    VersionNotFound { version: Version, cpu_arch: CPUArch },
    #[error("Version already installed at {:?}", path)]
    VersionAlreadyInstalled { path: PathBuf },
}

#[cfg(unix)]
fn filename_for_version(version: &Version, cpu_arch: CPUArch, ext: &str) -> String {
    format!(
        "node-{node_ver}-{platform}-{cpu_arch}.{ext}",
        node_ver = &version,
        platform = crate::system_info::platform_name(),
        cpu_arch = cpu_arch,
        ext = ext
    )
}

#[cfg(windows)]
fn filename_for_version(version: &Version, cpu_arch: CPUArch, ext: &str) -> String {
    format!(
        "node-{node_ver}-win-{cpu_arch}.{ext}",
        node_ver = &version,
        cpu_arch = cpu_arch,
        ext = ext,
    )
}

fn download_url(base_url: &Url, version: &Version, cpu_arch: CPUArch, ext: &str) -> Url {
    Url::parse(&format!(
        "{}/{}/{}",
        base_url.as_str().trim_end_matches('/'),
        version,
        filename_for_version(version, cpu_arch, ext)
    ))
    .unwrap()
}

/// Install a Node package
pub fn install_node_dist<P: AsRef<Path>>(
    version: &Version,
    node_dist_mirror: &Url,
    installations_dir: P,
    cpu_arch: CPUArch,
    show_loaders: bool,
) -> Result<(), Error> {
    let installation_dir = PathBuf::from(installations_dir.as_ref()).join(version.v_str());

    if installation_dir.exists() {
        return Err(Error::VersionAlreadyInstalled {
            path: installation_dir,
        });
    }

    std::fs::create_dir_all(installations_dir.as_ref())?;

    let temp_installations_dir = installations_dir.as_ref().join(".downloads");
    std::fs::create_dir_all(&temp_installations_dir)?;

    let portal = DirectoryPortal::new_in(&temp_installations_dir, installation_dir);

    for extract in Archive::supported() {
        let ext = extract.file_extension();
        let url = download_url(node_dist_mirror, version, cpu_arch, ext);
        debug!("Going to call for {}", &url);
        let response = crate::http::get(url.as_str())?;

        if !response.status().is_success() {
            continue;
        }

        debug!("Extracting response...");
        if show_loaders {
            extract.extract_pressed_into(
                portal.as_ref(),
                ResponseProgress::new(response, ProgressDrawTarget::stderr()),
            )?;
        } else {
            extract.extract_pressed_into(portal.as_ref(), response)?;
        }
        debug!("Extraction completed");

        let installed_directory = std::fs::read_dir(&portal)?
            .next()
            .ok_or(Error::EmptyTape)??;
        let installed_directory = installed_directory.path();

        let renamed_installation_dir = portal.join("installation");
        std::fs::rename(installed_directory, renamed_installation_dir)?;

        portal.teleport()?;

        return Ok(());
    }

    Err(Error::VersionNotFound {
        version: version.clone(),
        cpu_arch,
    })
}
