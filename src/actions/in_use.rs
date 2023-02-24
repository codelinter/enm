use super::command::Command;
use crate::app_config::AppConfig;
use crate::version_now::{version_now, Error};

#[derive(clap::Parser, Debug)]
pub struct InUse {}

impl Command for InUse {
    type Error = Error;

    fn apply(self, config: &AppConfig) -> Result<(), Self::Error> {
        let version_string = match version_now(config)? {
            Some(ver) => ver.v_str(),
            None => "none".into(),
        };
        println!("{version_string}");
        Ok(())
    }
}
