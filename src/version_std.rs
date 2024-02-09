use crate::app_config::AppConfig;
use crate::version::Version;
use std::str::FromStr;

pub fn find_version_std(config: &AppConfig) -> Option<Version> {
    if let Ok(version_path) = config.version_std_dir().canonicalize() {
        let file_name = version_path.parent()?.file_name()?;
        Version::from_str(file_name.to_str()?).ok()?.into()
    } else {
        Some(Version::Alias("default".into()))
    }
}
