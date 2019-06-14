
use std::process::Command;

use log::debug;

use crate::error::{Result, BenchError};

fn output_into_string(mut command: Command) -> Result<String> {
    let cmd_str = format!("{:?}", command);
    debug!("executing: {}", cmd_str);
    let output = command.output()?;
    match output.status.code() {
        Some(0) => {
            Ok(String::from_utf8(output.stdout)?)
        },
        Some(127) => {
            Err(BenchError::NoSuchCommand(cmd_str))
        },
        Some(code) => {
            Err(BenchError::CommandFailed(cmd_str, code))
        },
        None => {
            Err(BenchError::CommandKilledBySingal(cmd_str))
        } 
    }
}

/// Returns: shown messages
pub fn ssh(user_name: &str, ip: &str, remote_cmd: &str) -> Result<String> {
    let mut command = Command::new("ssh");
    command.arg(format!("{}@{}", user_name, ip)).arg(remote_cmd);

    output_into_string(command).map_err(|e| e.as_remote_if_possible(ip))
}

pub fn scp(is_dir: bool, user_name: &str, ip: &str, local_path: &str, remote_path: &str) -> Result<String> {
    let mut command = Command::new("scp");

    if is_dir {
        command.arg("-r");
    }
    
    command.arg(local_path);
    command.arg(format!("{}@{}:{}", user_name, ip, remote_path));

    match output_into_string(command).map_err(|e| e.as_remote_if_possible(ip)) {
        Err(BenchError::CommandFailedOnRemote(_, _, 2)) =>
            Err(BenchError::FileNotFound(local_path.to_owned())),
        other => other
    }
}

pub fn ls(path: &str) -> Result<String> {
    let mut command = Command::new("ls");
    command.arg(path);

    match output_into_string(command) {
        Err(BenchError::CommandFailed(_, 2)) =>
            Err(BenchError::FileNotFound(path.to_owned())),
        other => other
    }
}