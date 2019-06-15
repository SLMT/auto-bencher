
use std::path::{Path, PathBuf};
use std::fs;

use colored::*;
use log::info;
use clap::{ArgMatches, Arg, App, SubCommand};

use crate::error::{Result, BenchError};
use crate::config::Config;
use crate::parameters::{Parameter, ParameterList};
use crate::properties::PropertiesFileMap;
use crate::command;

const BENCH_DIR: &'static str = "benchmarker";
const PROP_DIR: &'static str = "props";

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

pub fn execute(config: &Config, args: &ArgMatches) -> Result<()> {
    let bench_type = args.value_of("BENCH TYPE").unwrap();
    let number_of_machine = args.value_of("NUMBER OF MACHINES").unwrap();
    let number_of_machine: usize = number_of_machine.parse()?;

    // Check number of machines
    if number_of_machine > config.machines.servers.len() {
        let avail = config.machines.servers.len();
        let err_msg = if avail == 1 {
            format!("There are only 1 server")
        } else {
            format!("There are only {} servers", avail)
        };
        return Err(BenchError::Message(err_msg));
    }
    
    info!("Start loading {} benchmarks on {} servers.",
            bench_type.cyan(), number_of_machine.to_string().cyan());

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
    let parameter = &param_list[0];

    generate_bench_dir(parameter)?;

    // Send all the prepared files to remote machines

    // Start the loading process

    // Check if any error shows up

    // Check if the loading finishes

    // Show the final result (where is the database, the size...)

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

fn generate_bench_dir(parameter: &Parameter) -> Result<()> {
    info!("Generating the benchmarker directory");

    // Ensure the existance of the benchmarker dir
    fs::create_dir_all(BENCH_DIR)?;

    // Copy the jar files to the benchmark dir
    if let Some(dirname) = parameter.get_basic_param("jar.dir") {
        copy_jars(dirname)?;
    } else {
        return Err(BenchError::Message(
            "No \"jar.dir\" is provided in the parameter file".to_owned()
        ))
    }

    // Read the default properties
    let mut map = PropertiesFileMap::from_dir(&Path::new("properties"))?;

    // Generate the properties files to the benchmark dir
    parameter.override_properties(&mut map);
    let prop_dir = Path::new(BENCH_DIR).join(PROP_DIR);
    map.output_to_dir(&prop_dir)?;

    Ok(())
}

fn copy_jars(dir_name: &str) -> Result<()> {
    let mut dir_path = PathBuf::new();
    dir_path.push("jars");
    dir_path.push(dir_name);

    let filenames = vec!["server.jar", "client.jar"];
    for filename in filenames {
        let jar_path = dir_path.join(filename);
        // Check if the jar exists
        command::ls(jar_path.to_str().unwrap())?;
        // Copy it to the benchmarker directory
        command::cp(false, jar_path.to_str().unwrap(), BENCH_DIR)?;
    }

    Ok(())
}