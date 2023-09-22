use crate::arch::Arch;
use crate::p_folders::Directories;
use crate::llevel::LogLevel;
use crate::path_ext::PathExt;
use crate::cd_file_changer::VersionFileStrategy;
use url::Url;

#[derive(clap::Parser, Debug)]
pub struct EnmConfig {
    /// <https://nodejs.org/dist/> mirror
    #[clap(
        long,
        env = "ENM_NODE_DIST_MIRROR",
        default_value = "https://nodejs.org/dist",
        global = true,
        hide= true,
        hide_env_values = true
    )]
    pub node_dist_mirror: Url,

    /// The root directory of enm installations.
    #[clap(
        long = "enm-dir",
        env = "ENM_DIR",
        global = true,
        hide_env_values = true
    )]
    pub base_dir: Option<std::path::PathBuf>,

    /// Where the current node version link is stored.
    /// This value will be populated automatically by evaluating
    /// `enm env` in your shell profile. Read more about it using `enm help env`
    #[clap(long, env = "ENM_RUNWAY_PATH", hide_env_values = true, hide = true)]
    runway_path: Option<std::path::PathBuf>,

    /// The log level of enm commands
    #[clap(
        long,
        env = "ENM_LOGLEVEL",
        default_value_t,
        global = true,
        hide=true,
        hide_env_values = true
    )]
    llevel: LogLevel,

    /// Override the architecture of the installed Node binary.
    /// Defaults to arch of enm binary.
    #[clap(
        long,
        env = "ENM_ARCH",
        default_value_t,
        hide=true,
        global = true,
        hide_env_values = true,
        hide_default_value = true
    )]
    pub arch: Arch,

    /// What file name to use to detect node version. Used whenever `enm use` or `enm install` is
    /// called without a version, or when `--detect-node` is configured on evaluation.
    #[clap(
        long,
        env = "ENM_CD_FILE_CHANGER",
        default_value_t,
        hide = true,
        global = true,
        hide_env_values = true
    )]
    cd_file_changer: VersionFileStrategy,

    /// Enable corepack support for each new installation.
    /// This will make enm call `corepack enable` on every Node.js installation.
    /// For more information about corepack see <https://nodejs.org/api/corepack.html>
    #[clap(
        long,
        env = "ENM_COREPACK_ENABLED",
        global = true,
        hide = true,
        hide_env_values = true
    )]
    corepack_enabled: bool,

    /// Resolve `engines.node` field in `package.json` whenever a `.node-version` or `.nvmrc` file is not present.
    /// Experimental: This feature is subject to change.
    /// Note: `engines.node` can be any semver range, with the latest satisfying version being resolved.
    #[clap(
        long,
        env = "ENM_RESOLVE_ENGINES",
        global = true,
        hide = true,
        hide_env_values = true,
        verbatim_doc_comment
    )]
    resolve_engines: bool,

    #[clap(skip)]
    p_folders: Directories,
}

impl Default for EnmConfig {
    fn default() -> Self {
        Self {
            node_dist_mirror: Url::parse("https://nodejs.org/dist/").unwrap(),
            base_dir: None,
            runway_path: None,
            llevel: LogLevel::Info,
            arch: Arch::default(),
            cd_file_changer: VersionFileStrategy::default(),
            corepack_enabled: false,
            resolve_engines: false,
            p_folders: Directories::default(),
        }
    }
}

impl EnmConfig {
    pub fn cd_file_changer(&self) -> VersionFileStrategy {
        self.cd_file_changer
    }

    pub fn corepack_enabled(&self) -> bool {
        self.corepack_enabled
    }

    pub fn resolve_engines(&self) -> bool {
        self.resolve_engines
    }

    pub fn runway_path(&self) -> Option<&std::path::Path> {
        match &self.runway_path {
            None => None,
            Some(v) => Some(v.as_path()),
        }
    }

    pub fn llevel(&self) -> LogLevel {
        self.llevel
    }

    pub fn base_dir_with_default(&self) -> std::path::PathBuf {
        if let Some(dir) = &self.base_dir {
            return dir.clone();
        }

        self.p_folders.default_base_dir()
    }

    pub fn installations_dir(&self) -> std::path::PathBuf {
        self.base_dir_with_default()
            .join("node-versions")
            .ensure_exists_silently()
    }

    pub fn default_version_dir(&self) -> std::path::PathBuf {
        self.vsetteres_dir().join("default")
    }

    pub fn vsetteres_dir(&self) -> std::path::PathBuf {
        self.base_dir_with_default()
            .join("vsetteres")
            .ensure_exists_silently()
    }

    pub fn runway_storage(&self) -> std::path::PathBuf {
        self.p_folders.runway_storage()
    }

    #[cfg(test)]
    pub fn with_base_dir(mut self, base_dir: Option<std::path::PathBuf>) -> Self {
        self.base_dir = base_dir;
        self
    }
}
