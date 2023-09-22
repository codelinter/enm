use crate::commands;
use crate::commands::command::Command;
use crate::config::EnmConfig;
use clap::Parser;

#[derive(clap::Parser, Debug)]
pub enum SubCommand {
    /// Show all remote Node.js versions
    #[clap(name = "show-remote", bin_name = "show-remote", visible_aliases = &["ll"])]
    LsRemote(commands::ls_remote::LsRemote),

    /// Show all locally installed Node.js versions
    #[clap(name = "show-local", bin_name = "show-local", visible_aliases = &["ls"])]
    LsLocal(commands::ls_local::LsLocal),

    /// Install a new Node.js version
    #[clap(name = "install", bin_name = "install", visible_aliases = &["i"])]
    Install(commands::install::Install),

    /// Use (or change to) specific Node.js version
    #[clap(name = "use", bin_name = "use")]
    Use(commands::r#use::Use),

    /// Print and set up required environment variables for enm
    ///
    /// This command generates a series of shell commands that
    /// should be evaluated by your shell to create a enm-ready environment.
    ///
    /// Each shell has its own syntax of evaluating a dynamic expression.
    /// For example, evaluating enm on Bash and Zsh would look like `eval "$(enm env)"`.
    #[clap(name = "env", bin_name = "env")]
    Env(commands::env::Env),

    /// Set a version as the default version
    #[clap(name = "default", bin_name = "default")]
    Default(commands::default::Default),

    /// Print the current Node.js version
    #[clap(name = "current", bin_name = "current")]
    Current(commands::current::Current),

    // /// Run a command within enm context
    // ///
    // /// Example:
    // /// --------
    // /// enm exec --using=v12.0.0 node --version
    // /// => v12.0.0
    // #[clap(name = "exec", bin_name = "exec", verbatim_doc_comment)]
    // Exec(commands::exec::Exec),

    /// Uninstall a Node.js version
    ///
    /// > Warning: when providing an vsetter, it will remove the Node version the vsetter
    /// is pointing to, along with the other vsetteres that point to the same version.
    #[clap(name = "uninstall", bin_name = "uninstall", visible_aliases = &["uni"])]
    Uninstall(commands::uninstall::Uninstall),
}

impl SubCommand {
    pub fn call(self, config: EnmConfig) {
        match self {
            Self::LsLocal(cmd) => cmd.call(config),
            Self::LsRemote(cmd) => cmd.call(config),
            Self::Install(cmd) => cmd.call(config),
            Self::Env(cmd) => cmd.call(config),
            Self::Use(cmd) => cmd.call(config),
            Self::Default(cmd) => cmd.call(config),
            Self::Current(cmd) => cmd.call(config),
            Self::Uninstall(cmd) => cmd.call(config),
        }
    }
}

/// Easy Node.js manager.
#[derive(clap::Parser, Debug)]
#[clap(name = "enm", version = env!("CARGO_PKG_VERSION"), bin_name = "enm")]
pub struct Cli {
    #[clap(flatten)]
    pub config: EnmConfig,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

pub fn parse() -> Cli {
    Cli::parse()
}
