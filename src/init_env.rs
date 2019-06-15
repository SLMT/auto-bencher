
use clap::{ArgMatches, App, SubCommand};
use log::{info, trace};
use colored::*;

use crate::error::{Result, BenchError};
use crate::config::Config;
use crate::command;

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init-env")
                .about("initializes the environment of all machines")
}

pub fn execute(config: &Config, _: &ArgMatches) -> Result<()> {
    println!("Starts initializing the environment");

    // Check local files
    if !check_local_jdk(config)? {
        panic!("cannot find the JDK at {}", config.path.local_jdk_path);
    }

    // For all the nodes
    for ip in &config.machines.all {
        print!("Node {} ...", ip);

        // Check the working directory
        if !check_working_dir(&config, ip)? {
            create_working_dir(&config, ip)?
        }

        // Check Java Runtime
        if !check_java_runtime(&config, ip)? {
            send_jdk(config, ip)?;
            unpack_jdk(config, ip)?;
            remove_jdk_package(config, ip)?;
        }

        println!("{}", "checked".green());
    }
    
    Ok(())
}

fn check_working_dir(config: &Config, ip: &str) -> Result<bool> {
    let cmd = format!("ls -l {}", config.path.remote_work_dir);

    match command::ssh(&config.system.user_name, ip, &cmd) {
        Err(BenchError::CommandFailedOnRemote(_, _, code, _)) if code == 2 => {
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

fn create_working_dir(config: &Config, ip: &str) -> Result<()> {
    info!("creating a working directory on {}", ip);

    let cmd = format!("mkdir -p {}", config.path.remote_work_dir);
    command::ssh(&config.system.user_name, ip, &cmd)
        .map(|out| trace!("mkdir: {}", out))
}

fn check_java_runtime(config: &Config, ip: &str) -> Result<bool> {
    info!("checking java runtime on {}", ip);

    let cmd = format!("{}/{}/bin/java -version", config.path.remote_work_dir, config.path.jdk_dir);

    // Check if the java is installed
    match command::ssh(&config.system.user_name, ip, &cmd) {
        Err(BenchError::NoSuchCommand(_)) => {
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

fn check_local_jdk(config: &Config) -> Result<bool> {
    info!("checking local jdk: {}", config.path.local_jdk_path);

    match command::ls(&config.path.local_jdk_path) {
        Ok(_) => Ok(true),
        Err(BenchError::FileNotFound(_)) => Ok(false),
        Err(e) => Err(e)
    }
}


fn send_jdk(config: &Config, ip: &str) -> Result<()> {
    info!("sending JDK to {}", ip);

    command::scp(false, &config.system.user_name, ip, 
            &config.path.local_jdk_path, &config.path.remote_work_dir)?;
    Ok(())
}

fn unpack_jdk(config: &Config, ip: &str) -> Result<()> {
    info!("unpacking {} on {}", config.path.jdk_package, ip);
    
    let cmd = format!("tar -C {} -zxf {}/{}", config.path.remote_work_dir, 
            config.path.remote_work_dir, config.path.jdk_package);
    command::ssh(&config.system.user_name, ip, &cmd)?;
    Ok(())
}

fn remove_jdk_package(config: &Config, ip: &str) -> Result<()> {
    info!("removing {} on {}", config.path.jdk_package, ip);
    
    let cmd = format!("rm {}/{}", config.path.remote_work_dir, 
            config.path.jdk_package);
    command::ssh(&config.system.user_name, ip, &cmd)?;
    Ok(())
}