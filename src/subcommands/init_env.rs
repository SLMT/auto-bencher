
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
    info!("Starts initializing the environment");

    // Check local files
    if !check_local_jdk(config)? {
        return Err(BenchError::Message(
            format!("cannot find the JDK at {}", config.jdk.package_path)
        ));
    }

    // For all the nodes
    for ip in &config.machines.all {
        info!("Checking node '{}' ...", ip);

        // Create the working directory
        create_working_dir(&config, ip)?;

        // Check Java Runtime
        if !check_java_runtime(&config, ip)? {
            send_jdk(config, ip)?;
            unpack_jdk(config, ip)?;
            remove_jdk_package(config, ip)?;
        }

        info!("Node '{}' {}", ip, "checked".green());
    }
    
    Ok(())
}

fn check_working_dir(config: &Config, ip: &str) -> Result<bool> {
    let cmd = format!("ls -l {}", config.system.remote_work_dir);

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
    info!("Creating a working directory on {}", ip);

    for dir in ["databases", "results"].iter() {
        let cmd = format!("mkdir -p {}/{}",
            config.system.remote_work_dir, dir);
        command::ssh(&config.system.user_name, ip, &cmd)
            .map(|out| trace!("mkdir: {}", out))?;
    }
    Ok(())
}

fn check_java_runtime(config: &Config, ip: &str) -> Result<bool> {
    info!("Checking java runtime on {}", ip);

    let cmd = format!("{}/{}/bin/java -version",
        config.system.remote_work_dir, config.jdk.dir_name);

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
    info!("Checking local jdk: {}", config.jdk.package_path);

    match command::ls(&config.jdk.package_path) {
        Ok(_) => Ok(true),
        Err(BenchError::FileNotFound(_)) => Ok(false),
        Err(e) => Err(e)
    }
}


fn send_jdk(config: &Config, ip: &str) -> Result<()> {
    info!("Sending JDK to {}", ip);

    command::scp_to(false, &config.system.user_name, ip, 
            &config.jdk.package_path, &config.system.remote_work_dir)?;
    Ok(())
}

fn unpack_jdk(config: &Config, ip: &str) -> Result<()> {
    info!("Unpacking {} on {}", config.jdk.package_filename, ip);
    
    let cmd = format!("tar -C {} -zxf {}/{}", config.system.remote_work_dir, 
            config.system.remote_work_dir, config.jdk.package_filename);
    command::ssh(&config.system.user_name, ip, &cmd)?;
    Ok(())
}

fn remove_jdk_package(config: &Config, ip: &str) -> Result<()> {
    info!("Removing {} on {}", config.jdk.package_filename, ip);
    
    let cmd = format!("rm {}/{}", config.system.remote_work_dir, 
            config.jdk.package_filename);
    command::ssh(&config.system.user_name, ip, &cmd)?;
    Ok(())
}