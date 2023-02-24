use crate::actions;
use crate::actions::command::Command;
use crate::app_config::AppConfig;
use clap::Parser;

#[derive(clap::Parser, Debug)]
pub enum Evaluator {
    /// Show all remote NodeJS versions
    #[clap(name = "show-remote", bin_name = "show-remote", visible_aliases = &["sr"])]
    ShowRemote(actions::ls_remote::ShowRemote),

    /// Show all locally installed NodeJS versions
    #[clap(name = "show-local", bin_name = "show-local", visible_aliases = &["sl"])]
    ShowLocal(actions::ls_local::ShowLocal),

    /// Install a provided NodeJS version
    #[clap(name = "install", bin_name = "install", visible_aliases = &["i"])]
    Install(actions::firstrun::FirstRun),

    /// Switch to provided NodeJS version
    /// 
    /// Ex: Switch to latest NodeJS version 18 
    /// `enm switch 18`
    #[clap(name = "switch", bin_name = "switch")]
    Switch(actions::switch::Switch),

    /// Depose environment variables for enm
    ///
    /// Ex: `eval "$(enm source)"`.
    #[clap(name = "source", bin_name = "source")]
    AppVan(actions::app_van::AppVan),

    /// Set provided version as the default NodeJS version
    ///
    #[clap(name = "default", bin_name = "default")]
    Default(actions::default::Default),

    /// Print the NodeJS version currenly in use
    #[clap(name = "inuse", bin_name = "inuse")]
    InUse(actions::in_use::InUse),

    /// Run commands under a specific node version
    ///
    /// Ex: Run commands under NodeJS version 18 
    /// enm run --with=v18 node --version
    /// => v18.20.12
    #[clap(name = "run", bin_name = "run", verbatim_doc_comment)]
    RunX(actions::runx::RunX),
    
    /// Uninstall provided NodeJS version
    ///
    /// Ex: Uninstall NodeJS version 16 
    /// enm uninstall 18
    #[clap(name = "uninstall", bin_name = "uninstall", visible_aliases = &["ui"])]
    Uninstall(actions::uninstall::Uninstall),
}

impl Evaluator {
    pub fn call(self, config: AppConfig) {
        match self {
            Self::ShowLocal(cmd) => cmd.call(config),
            Self::ShowRemote(cmd) => cmd.call(config),
            Self::Install(cmd) => cmd.call(config),
            Self::AppVan(cmd) => cmd.call(config),
            Self::Switch(cmd) => cmd.call(config),
            Self::Default(cmd) => cmd.call(config),
            Self::InUse(cmd) => cmd.call(config),
            Self::RunX(cmd) => cmd.call(config),
            Self::Uninstall(cmd) => cmd.call(config),
        }
    }
}

/// Easy NodeJS (version) manager (ENM).
#[derive(clap::Parser, Debug)]
#[clap(name = "enm", version = env!("CARGO_PKG_VERSION"), bin_name = "enm")]
pub struct Cli {
    #[clap(flatten)]
    pub app_cfg: AppConfig,
    #[clap(subcommand)]
    pub valuator: Evaluator,
}

pub fn parse() -> Cli {
    Cli::parse()
}
