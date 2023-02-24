use crate::app_config::AppConfig;
use crate::outln;
use colored::Colorize;

pub trait Command: Sized {
    type Error: std::error::Error;
    fn apply(self, config: &AppConfig) -> Result<(), Self::Error>;

    fn handle_error(err: Self::Error, config: &AppConfig) {
        let err_s = format!("{err}");
        outln!(config, Error, "{} {}", "error:".red().bold(), err_s.red());
        std::process::exit(1);
    }

    fn call(self, config: AppConfig) {
        match self.apply(&config) {
            Ok(()) => (),
            Err(err) => Self::handle_error(err, &config),
        }
    }
}
