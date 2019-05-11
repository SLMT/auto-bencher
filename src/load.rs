
use colored::*;
use clap::{ArgMatches, Arg, App, SubCommand};

use crate::error::BenchError;
use crate::config::Config;

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("load")
                .arg(Arg::with_name("BENCH TYPE")
                    .help("Sets the type of the benchmark")
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("NUMBER OF MACHINES")
                    .help("Sets the number of machines to be loaded")
                    .required(true)
                    .index(2))
                .about("loads the testbed of the specified benchmark for the given number of machines")
}

pub fn execute(config: &Config, args: &ArgMatches) -> Result<(), BenchError> {
    let bench_type = args.value_of("BENCH TYPE").unwrap();
    let number_of_machine = args.value_of("NUMBER OF MACHINES").unwrap();
    let number_of_machine: usize = number_of_machine.parse()
            .map_err(|e| BenchError::throw("parsing # of machines fails", e))?;

    // Check number of machines
    if number_of_machine > config.machines.servers.len() {
        let avail = config.machines.servers.len();
        let err_msg = if avail == 1 {
            format!("There are only 1 server")
        } else {
            format!("There are only {} servers", avail)
        };
        return Err(BenchError::message(&err_msg));
    }
    
    println!("Start loading {} benchmarks on {} servers.",
            bench_type.cyan(), number_of_machine.to_string().cyan());

    // Prepare loading binary

    // Prepare properties files

    // Send all the prepared files to remote machines

    // Start the loading process

    Ok(())
}