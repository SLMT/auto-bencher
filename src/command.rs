
use std::process::Command;

use log::debug;

use crate::error::BenchError;

/// Returns: shown messages
pub fn ssh(user_name: &str, ip: &str, cmd: &str) -> Result<String, BenchError> {
    let result = Command::new("ssh")
            .arg(format!("{}@{}", user_name, ip))
            .arg(cmd)
            .output()
            .map_err(|e| BenchError::throw("execute command fails", e))?;

    debug!("executing: ssh {}@{} '{}'", user_name, ip, cmd);
    
    match result.status.code() {
        Some(127) => {
            return Err(BenchError::NoSuchCommand);
        },
        Some(0) => { },
        Some(code) => {
            return Err(BenchError::CommandFails(code));
        },
        None => {
            return Err(BenchError::message("the command terminates by a signal"));
        }
    }

    let output = String::from_utf8(result.stdout)
            .map_err(|e| BenchError::throw("parsing command output fails", e))?;

    Ok(output)
}

pub fn scp(is_dir: bool, user_name: &str, ip: &str, local_path: &str, remote_path: &str) -> Result<String, BenchError> {
    let mut command = Command::new("scp");

    if is_dir {
        command.arg("-r");
        debug!("executing: scp -r {} {}@{}:{}", local_path, user_name, ip, remote_path);
    } else {
        debug!("executing: scp {} {}@{}:{}", local_path, user_name, ip, remote_path);
    }
    
    command.arg(local_path);
    command.arg(format!("{}@{}:{}", user_name, ip, remote_path));

    let output = command.output().map_err(|e| BenchError::throw("execute command fails", e))?;

    match output.status.code() {
        Some(0) => {
            Ok(String::from_utf8(output.stdout)
                    .map_err(|e| BenchError::throw("parsing command output fails", e))?)
        },
        Some(2) => {
            Err(BenchError::FileNotFound)
        },
        Some(code) => {
            Err(BenchError::CommandFails(code))
        },
        None => {
            Err(BenchError::message("the command terminates by a signal"))
        } 
    }
}

pub fn ls(path: &str) -> Result<String, BenchError> {
    let output = Command::new("ls").arg(path)
            .output().map_err(|e| BenchError::throw("executes ls fails", e))?;

    debug!("executing: ls {}", path);
    
    match output.status.code() {
        Some(0) => {
            Ok(String::from_utf8(output.stdout)
                    .map_err(|e| BenchError::throw("parsing command output fails", e))?)
        },
        Some(2) => {
            Err(BenchError::FileNotFound)
        },
        Some(code) => {
            Err(BenchError::CommandFails(code))
        },
        None => {
            Err(BenchError::message("the command terminates by a signal"))
        } 
    }
}