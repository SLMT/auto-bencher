
use std::path::PathBuf;

use colored::*;
use log::*;
use clap::{ArgMatches, Arg, App, SubCommand};

use crate::error::{Result, BenchError};
use crate::config::Config;
use crate::parameters::ParameterList;
use crate::connections::Action;

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("load")
                .arg(Arg::with_name("BENCH TYPE")
                    .help("Sets the type of the benchmark")
                    .required(true)
                    .index(1))
                .about("loads the testbed of the specified benchmark")
}

pub fn execute(config: &Config, args: &ArgMatches) -> Result<()> {
    let bench_type = args.value_of("BENCH TYPE").unwrap();
    
    info!("Preparing for loading {} benchmarks...",
        bench_type.cyan());

    // Read the parameter file
    let param_list = read_parameter_file(bench_type)?;

    // The file should only produce single "Parameter"
    let param_list = param_list.to_vec();
    if param_list.len() > 1 {
        return Err(BenchError::Message(format!(
            "{}'s parameter file contains more than one combination", 
            bench_type.cyan()
        )))
    }

    super::run_server_and_client(config, &param_list[0],
        &bench_type, Action::Loading)?;

    // Show the final result (where is the database, the size...)
    info!("Loading of benchmark {} finished.", bench_type);

    Ok(())
}

fn read_parameter_file(bench_type: &str) -> Result<ParameterList> {
    let mut param_file = PathBuf::new();
    param_file.push("parameters");
    param_file.push("loading");
    param_file.push(bench_type);
    param_file.set_extension("toml");

    info!("Reading the parameter file: '{}'", param_file.to_str().unwrap());

    ParameterList::from_file(&param_file)
}
