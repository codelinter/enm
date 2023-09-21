use super::vsetter::VSetter;
use super::command::Command;
use crate::config::EnmConfig;
use crate::user_version::UserVersion;

#[derive(clap::Parser, Debug)]
pub struct Default {
    version: UserVersion,
}

impl Command for Default {
    type Error = super::vsetter::Error;

    fn apply(self, config: &EnmConfig) -> Result<(), Self::Error> {
        VSetter {
            name: "default".into(),
            to_version: self.version,
        }
        .apply(config)
    }

    fn handle_error(err: Self::Error, config: &EnmConfig) {
        VSetter::handle_error(err, config);
    }
}
