use super::command::Command;
use crate::config::EnmConfig;
use crate::version_now::{version_now, Error};

#[derive(clap::Parser, Debug)]
pub struct Current {}

impl Command for Current {
    type Error = Error;

    fn apply(self, config: &EnmConfig) -> Result<(), Self::Error> {
        let version_string = match version_now(config)? {
            Some(ver) => ver.v_str(),
            None => "none".into(),
        };
        println!("{version_string}");
        Ok(())
    }
}
