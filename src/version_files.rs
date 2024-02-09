use crate::app_config::AppConfig;
use crate::sift_method::SiftMethod;
use crate::user_version::UserVersion;
use crate::version_std;
use encoding_rs_io::DecodeReaderBytes;
use log::info;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

const PATH_PARTS: [&str; 3] = [".nvmrc", ".node-version", "package.json"];

pub fn get_user_version_for_directory(
    path: impl AsRef<Path>,
    config: &AppConfig,
) -> Option<UserVersion> {
    match config.sift_method() {
        SiftMethod::Local => get_user_version_for_single_directory(path),
        SiftMethod::Recursive => {
            get_user_version_for_directory_recursive(path).or_else(|| {
                info!("Did not find anything recursively. Falling back to default alias.");
                version_std::find_version_std(config).map(UserVersion::Full)
            })
        }
    }
}

fn get_user_version_for_directory_recursive(
    path: impl AsRef<Path>,
) -> Option<UserVersion> {
    let mut current_path = Some(path.as_ref());

    while let Some(child_path) = current_path {
        if let Some(version) = get_user_version_for_single_directory(child_path) {
            return Some(version);
        }

        current_path = child_path.parent();
    }

    None
}

fn get_user_version_for_single_directory(
    path: impl AsRef<Path>,
) -> Option<UserVersion> {
    let path = path.as_ref();

    for path_part in &PATH_PARTS {
        let new_path = path.join(path_part);
        info!(
            "Looking for version file in {}. exists? {}",
            new_path.display(),
            new_path.exists()
        );
        if let Some(version) = get_user_version_for_file(&new_path) {
            return Some(version);
        }
    }

    None
}

pub fn get_user_version_for_file(
    path: impl AsRef<Path>,
) -> Option<UserVersion> {
    let is_pkg_json = match path.as_ref().file_name() {
        Some(name) => name == "package.json",
        None => false,
    };
    let file = std::fs::File::open(path).ok()?;
    let file = {
        let mut reader = DecodeReaderBytes::new(file);
        let mut version = String::new();
        reader.read_to_string(&mut version).map(|_| version)
    };

    match (file, is_pkg_json) {
        (_, true) => None,
        (Err(err), _) => {
            info!("Unable to read file: {}", err);
            None
        }
        (Ok(version), false) => {
            info!("Found string {:?} in version file", version);
            UserVersion::from_str(version.trim()).ok()
        }
    }
}
