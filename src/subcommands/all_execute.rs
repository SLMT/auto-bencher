
use log::*;
use clap::{ArgMatches, Arg, App, SubCommand};

use crate::error::Result;
use crate::config::Config;
use crate::command;

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("all-exec")
                .arg(Arg::with_name("COMMAND")
                    .help("The command to execute on all the machines")
                    .required(true)
                    .index(1))
                .about("executes the given command on all the machines")
}

pub fn execute(config: &Config, args: &ArgMatches) -> Result<()> {
    let cmd = args.value_of("COMMAND").unwrap();

    for ip in &config.machines.all {
        info!("Executing the command on {}", &ip);
        let output = command::ssh(&config.system.user_name, &ip, &cmd)?;
        println!("{}", output);
    }

    Ok(())
}