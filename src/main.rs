
mod error;
mod config;
mod command;
mod init_env;
mod load;
mod parameters;
mod properties;

use clap::{Arg, ArgMatches, App};
use colored::*;
use log::error;

use error::BenchError;
use config::Config;

fn main() {
    // Setup the logger
    set_logger_level();
    pretty_env_logger::init();

    let matches = App::new("Auto Bencher")
                       .version("1.0")
                       .author("Yu-Shan Lin <yslin@datalab.cs.nthu.edu.tw>")
                       .about("Automatically run benchmarking using VanillaBench or ElasqlBench")
                       .arg(Arg::with_name("config")
                            .short("c")
                            .long("config")
                            .value_name("FILE")
                            .help("Sets the path to a config file")
                            .takes_value(true))
                       .subcommand(init_env::get_sub_command())
                       .subcommand(load::get_sub_command())
                       .get_matches();
    
    match execute(matches) {
        Ok(_) => println!("Auto Bencher finishes."),
        Err(e) => error!("Auto Bencher exits with an {}:\n{}",
                "error".red(), e)
    }
}

fn set_logger_level() {
    match std::env::var("RUST_LOG") {
        Ok(_) => {},
        Err(_) => std::env::set_var("RUST_LOG", "INFO"),
    }
}

fn execute(matches: ArgMatches) -> Result<(), BenchError> {
     // Read the config
    let config_file_path = matches.value_of("config").unwrap_or("config.toml");
    let config = Config::from_file(&config_file_path)?;

    // Choose action according to the sub command
    if let Some(matches) = matches.subcommand_matches("init-env") {
        init_env::execute(&config, matches)?;
    } else if let Some(matches) = matches.subcommand_matches("load") {
        let map = properties::PropertiesFileMap::from_dir(&std::path::Path::new("properties"))?;
        dbg!(map);
        // load::execute(&config, matches)?;
    }

    Ok(())
}