
use std::process::Command;

use clap::{ArgMatches, App, SubCommand};
use log::trace;
use colored::*;

use crate::error::BenchError;
use crate::config::Config;
use crate::command;

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init-env")
                .about("initializes the environment of all machines")
}

pub fn execute(config: &Config, args: &ArgMatches) -> Result<(), BenchError> {
    println!("Starts initializing the environment");

    // Check local files
    if !check_local_jdk(config)? {
        return Err(BenchError::Throw(
            format!("cannot find the JDK at {}", config.path.local_jdk_path)));
    }

    // For all the nodes
    for ip in &config.machines.all {
        print!("Node {} ...", ip);

        // Check the working directory
        if !check_working_dir(&config, ip)? {
            trace!("creating working dir for {}", ip);
            create_working_dir(&config, ip)?
        }

        // Check Java Runtime
        if !check_java_runtime(&config, ip)? {
            trace!("sending jdk for {}", ip);
            // TODO: check local jdk pack
            // TODO: send jdk
            // TODO: unpack jdk
        }

        println!("{}", "checked".green());
    }
    
    Ok(())
}

fn check_working_dir(config: &Config, ip: &str) -> Result<bool, BenchError> {
    let cmd = format!("ls -l {}", config.path.remote_work_dir);

    match command::ssh(&config.system.user_name, ip, &cmd) {
        Err(BenchError::CommandFails(code)) if code == 2 => {
            Ok(false)
        },
        Err(e) => {
            Err(e)
        },
        Ok(output) => {
            trace!("ls: {}", output);
            Ok(true)
        } 
    }
}

fn create_working_dir(config: &Config, ip: &str) -> Result<(), BenchError> {
    let cmd = format!("mkdir -p {}", config.path.remote_work_dir);

    command::ssh(&config.system.user_name, ip, &cmd)
        .map(|out| trace!("mkdir: {}", out))
}

fn check_java_runtime(config: &Config, ip: &str) -> Result<bool, BenchError> {
    let cmd = format!("{}/{}/bin/java", config.path.remote_work_dir, config.path.jdk_dir);

    // Check if the java is installed
    match command::ssh(&config.system.user_name, ip, &cmd) {
        Err(BenchError::NoSuchCommand) => {
            Ok(false)
        },
        Err(e) => {
            Err(e)
        },
        Ok(output) => {
            trace!("java: {}", output);
            Ok(true)
        } 
    }
}

fn check_local_jdk(config: &Config) -> Result<bool, BenchError> {
    match command::ls(&config.path.local_jdk_path) {
        Ok(_) => Ok(true),
        Err(BenchError::FileNotFound) => Ok(false),
        Err(e) => Err(e)
    }
}


fn send_jdk(config: &Config, ip: &str) -> Result<(), BenchError> {
    Ok(())
}