
use std::path::Path;

use colored::*;
use log::*;
use clap::{ArgMatches, Arg, App, SubCommand};

use crate::error::{Result, BenchError};
use crate::config::Config;
use crate::parameters::ParameterList;
use crate::connections::Action;

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("bench")
                .arg(Arg::with_name("BENCH TYPE")
                    .help("Sets the type of the benchmark")
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("PARAMETER FILE")
                    .help("The parameters of running the benchmarks")
                    .required(true)
                    .index(2))
                .about("running the benchmarks")
}

pub fn execute(config: &Config, args: &ArgMatches) -> Result<()> {
    let bench_type = args.value_of("BENCH TYPE").unwrap();
    let param_file = args.value_of("PARAMETER FILE").unwrap();
    
    info!("Preparing for running benchmarks...");
    info!("Using parameter file '{}'", param_file);

    // Read the parameter file
    let param_list = ParameterList::from_file(Path::new(param_file))?;
    let param_list = param_list.to_vec();
    for job_id in 0 .. param_list.len() {
        info!("Running job {}...", job_id);

        super::run_server_and_client(config, &param_list[job_id],
            &bench_type, Action::Benchmarking)?;

        info!("Job {} finished", job_id);
    }

    // Show the final result (where is the database, the size...)
    info!("Loading of benchmark {} finished.", bench_type);

    Ok(())
}