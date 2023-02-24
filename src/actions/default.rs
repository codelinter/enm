use super::alias::Alias;
use super::command::Command;
use crate::app_config::AppConfig;
use crate::user_version::UserVersion;

#[derive(clap::Parser, Debug)]
pub struct Default {
    version: UserVersion,
}

impl Command for Default {
    type Error = super::alias::Error;

    fn apply(self, config: &AppConfig) -> Result<(), Self::Error> {
        Alias {
            name: "default".into(),
            to_version: self.version,
        }
        .apply(config)
    }

    fn handle_error(err: Self::Error, config: &AppConfig) {
        Alias::handle_error(err, config);
    }
}
