use crate::cpu_arch::CPUArch;
use crate::ll_int::LLInt;
use crate::path_ext::PathExt;
use crate::sift_method::SiftMethod;
use crate::std_system_structure::StdStructure;
use url::Url;

#[derive(clap::Parser, Debug)]
pub struct AppConfig {
    #[clap(
        long,
        env = "ENM_NODE_DIST_MIRROR",
        default_value = "https://nodejs.org/dist",
        global = true,
        hide = true,
        hide_env_values = true
    )]
    pub node_dist_mirror: Url,

    /// Folder path where NodeJS versions will be installed.
    #[clap(
        long = "enm-dir",
        env = "ENM_DIR",
        hide = true,
        global = true,
        hide_env_values = true
    )]
    pub base_dir: Option<std::path::PathBuf>,

    /// Where the current node version link is stored.
    /// This value will be populated automatically by evaluating
    /// `enm source` in your shell profile. Read more about it using `enm help source`
    #[clap(long, env = "ENM_SHIM", hide_env_values = true, hide = true)]
    plural_ctx: Option<std::path::PathBuf>,

    /// Manage emitter verbosity
    #[clap(
        long,
        env = "ENM_LL_INT",
        default_value_t,
        hide = true,
        global = true,
        hide_env_values = true
    )]
    ll_int: LLInt,

    /// Override the cpu_architecture of the installed Node binary.
    /// Defaults to cpu_arch of enm binary.
    #[clap(
        long,
        env = "ENM_ARCH",
        default_value_t,
        global = true,
        hide = true,
        hide_env_values = true,
        hide_default_value = true
    )]
    pub cpu_arch: CPUArch,

    /// Method to resolve NodeJS version. Used whenever `enm switch` or `enm install` is
    /// called without a version, or when `--on-enter` is configured on evaluation.
    #[clap(
        long,
        env = "ENM_SIFT_METHOD",
        default_value_t,
        global = true,
        hide = true,
        hide_env_values = true
    )]
    sift_method: SiftMethod,

    /// Enable package manager support for enm
    /// allowing enm to call `corepack enable` on every NodeJS install.
    /// More details on corepack can be found at <https://nodejs.org/api/corepack.html>
    #[clap(
        long,
        env = "ENM_COREPACK_ENABLED",
        global = true,
        hide = true,
        hide_env_values = true
    )]
    corepack_enabled: bool,

    #[clap(skip)]
    std_system_structure: StdStructure,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            node_dist_mirror: Url::parse("https://nodejs.org/dist/").unwrap(),
            base_dir: None,
            plural_ctx: None,
            ll_int: LLInt::Info,
            cpu_arch: CPUArch::default(),
            sift_method: SiftMethod::default(),
            corepack_enabled: false,
            std_system_structure: StdStructure::default(),
        }
    }
}

impl AppConfig {
    pub fn sift_method(&self) -> SiftMethod {
        self.sift_method
    }

    pub fn corepack_enabled(&self) -> bool {
        self.corepack_enabled
    }

    pub fn plural_ctx(&self) -> Option<&std::path::Path> {
        match &self.plural_ctx {
            None => None,
            Some(v) => Some(v.as_path()),
        }
    }

    pub fn ll_int(&self) -> LLInt {
        self.ll_int
    }

    pub fn base_dir_with_default(&self) -> std::path::PathBuf {
        if let Some(dir) = &self.base_dir {
            return dir.clone();
        }

        self.std_system_structure.default_base_dir()
    }

    pub fn installations_dir(&self) -> std::path::PathBuf {
        self.base_dir_with_default()
            .join("node-versions")
            .ensure_exists_silently()
    }

    pub fn version_std_dir(&self) -> std::path::PathBuf {
        self.aliases_dir().join("default")
    }

    pub fn aliases_dir(&self) -> std::path::PathBuf {
        self.base_dir_with_default()
            .join("aliases")
            .ensure_exists_silently()
    }

    pub fn plural_ctx_storage(&self) -> std::path::PathBuf {
        self.std_system_structure.plural_ctx_storage()
    }
}
