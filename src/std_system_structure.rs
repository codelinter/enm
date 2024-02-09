use etcetera::BaseStrategy;
use std::path::PathBuf;

use crate::path_ext::PathExt;

fn xdg_dir(env: &str) -> Option<PathBuf> {
    if cfg!(windows) {
        let env_var = std::env::var_os(env)?;
        Some(PathBuf::from(env_var))
    } else {
        // On non-Windows platforms, `etcetera` already handles XDG variables
        None
    }
}

fn runtime_dir(basedirs: &impl BaseStrategy) -> Option<PathBuf> {
    xdg_dir("XDG_RUNTIME_DIR").or_else(|| basedirs.runtime_dir())
}

fn state_dir(basedirs: &impl BaseStrategy) -> Option<PathBuf> {
    xdg_dir("XDG_STATE_HOME").or_else(|| basedirs.state_dir())
}

fn cache_dir(basedirs: &impl BaseStrategy) -> PathBuf {
    xdg_dir("XDG_CACHE_HOME").unwrap_or_else(|| basedirs.cache_dir())
}

/// A helper struct for std_system_structure in enm that uses XDG Base Directory Specification
/// if applicable for the platform.
#[derive(Debug)]
pub struct StdStructure(
    #[cfg(windows)] etcetera::base_strategy::Windows,
    #[cfg(not(windows))] etcetera::base_strategy::Xdg,
);

impl Default for StdStructure {
    fn default() -> Self {
        Self(etcetera::choose_base_strategy().expect("choosing base strategy"))
    }
}

impl StdStructure {
    pub fn strategy(&self) -> &impl BaseStrategy {
        &self.0
    }

    pub fn default_base_dir(&self) -> PathBuf {
        let strategy = self.strategy();
        let modern = strategy.data_dir().join("enm");
        if modern.exists() {
            return modern;
        }

        let legacy = strategy.home_dir().join(".enm");
        if legacy.exists() {
            return legacy;
        }

        #[cfg(target_os = "macos")]
        {
            let basedirs = etcetera::base_strategy::Apple::new().expect("Unable to get home directory");
            let legacy = basedirs.data_dir().join("enm");
            if legacy.exists() {
                return legacy;
            }
        }

        modern.ensure_exists_silently()
    }

    pub fn plural_ctx_storage(&self) -> PathBuf {
        let basedirs = self.strategy();
        let dir = runtime_dir(basedirs)
            .or_else(|| state_dir(basedirs))
            .unwrap_or_else(|| cache_dir(basedirs));
        dir.join("enm_shim")
    }
}
