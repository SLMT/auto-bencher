
use std::path::Path;

use colored::*;
use log::*;
use clap::{ArgMatches, Arg, App, SubCommand};

use crate::error::{Result, BenchError};
use crate::config::Config;
use crate::parameters::ParameterList;
use crate::connections::Action;

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("load")
                .arg(Arg::with_name("DB NAME")
                    .help("The name of the database for holding testing data")
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("PARAMETER FILE")
                    .help("The parameters of running the loading program")
                    .required(true)
                    .index(2))
                .about("loads the testbed using the given parameters")
}

pub fn execute(config: &Config, args: &ArgMatches) -> Result<()> {
    let db_name = args.value_of("DB NAME").unwrap();
    let param_file = args.value_of("PARAMETER FILE").unwrap();
    
    info!("Preparing for loading testbed into '{}'...",
        db_name.cyan());
    info!("Using parameter file '{}'", param_file);

    // Read the parameter file
    let param_list = ParameterList::from_file(Path::new(param_file))?;

    // The file should only produce single "Parameter"
    let param_list = param_list.to_vec();
    if param_list.len() > 1 {
        return Err(BenchError::Message(format!(
            "The parameter file contains more than one combination"
        )));
    }

    super::run_server_and_client(config, &param_list[0],
        &db_name, Action::Loading, None)?;

    // Show the final result (where is the database, the size...)
    info!("Loading testbed finished.");

    Ok(())
}