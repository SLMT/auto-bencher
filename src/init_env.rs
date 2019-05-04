
use std::process::Command;

use clap::{ArgMatches, App, SubCommand};

use crate::config::Config;

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init-env")
                .about("initializes the environment of all machines")
}

pub fn execute(config: &Config, args: &ArgMatches) {
    // Check Java Runtime
    // let result = Command::new("ssh")
    //     .arg(format!("{}@{}", config.user_name, ))

    // Send JDK
}