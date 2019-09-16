
use log::*;

use crate::config::Config;
use crate::error::{Result, BenchError};
use crate::command;
use super::Action;
use super::ConnectionInfo;

pub struct Client {
    config: Config,
    connection_info: ConnectionInfo,
    vm_args: String,
}

impl Client {
    pub fn new(config: Config, connection_info: ConnectionInfo, vm_args: String) -> Client {
        Client {
            config,
            connection_info,
            vm_args
        }
    }

    pub fn send_bench_dir(&self) -> Result<()> {
        command::scp_to(
            true,
            &self.config.system.user_name,
            &self.connection_info.ip,
            "benchmarker",
            &self.config.system.remote_work_dir
        )?;
        Ok(())
    }

    pub fn clean_previous_results(&self) -> Result<()> {
        let cmd = format!("rm -r {}",
            self.result_path());
        let result = command::ssh(
            &self.config.system.user_name,
            &self.connection_info.ip,
            &cmd
        );
        match result {
            Err(BenchError::CommandFailedOnRemote(_, _, 1, _)) =>
                    info!("No previous results are found on '{}'", self.connection_info.ip),
            Err(e) => return Err(e),
            _ => {}
        }
        Ok(())
    }

    pub fn start(&self, action: Action) -> Result<()> {
        info!("Starting client {}...", self.id());
        // [client id] [action]
        let prog_args = format!("{} {}",
            self.connection_info.id, action.as_int());
        let cmd = format!("{} {} -jar {} {} > {} 2>&1 &",
            self.config.jdk.remote_java_bin,
            self.vm_args,
            self.jar_path(),
            prog_args,
            self.log_path()
        );
        command::ssh(
            &self.config.system.user_name,
            &self.connection_info.ip,
            &cmd
        )?;
        info!("Client {} is running.", self.id());
        Ok(())
    }

    pub fn check_for_finished(&self, action: Action) -> Result<bool> {
        let keyword = match action {
            Action::Loading => "loading procedure finished.",
            Action::Benchmarking => "benchmark process finished.",
        };

        if let Ok(output) = self.grep_log("Exception") {
            return Err(BenchError::Message(
                format!("Client {} error: {}", self.id(), output)));
        }

        if let Ok(output) = self.grep_log("error") {
            return Err(BenchError::Message(
                format!("Client {} error: {}", self.id(), output)));
        }

        match self.grep_log(keyword) {
            Ok(_) => Ok(true),
            Err(BenchError::CommandFailedOnRemote(_, _, 1, _)) =>
                Ok(false),
            Err(e) => Err(e)
        }
    }

    pub fn pull_log(&self) -> Result<()> {
        unimplemented!();
    }

    pub fn pull_csv(&self) -> Result<()> {
        let remote_result_path = format!("{}/*.csv",
            self.result_path());
        command::scp_from(
            false,
            &self.config.system.user_name,
            &self.connection_info.ip,
            &remote_result_path,
            "results"
        )?;
        Ok(())
    }

    pub fn get_total_throughput(&self) -> Result<u32> {
        let cmd = format!("grep 'TOTAL' {}/*",
            self.result_path()
        );
        let output = command::ssh(
            &self.config.system.user_name,
            &self.connection_info.ip,
            &cmd
        )?;
        // Output should be '...:TOTAL - committed: xxxx,...'
        let start = output.find("committed")
            .ok_or(BenchError::Message(
                format!("cannot parse result file: {}", output)
            ))? + 11;
        let end = output[start ..].find(",")
            .ok_or(BenchError::Message(
                format!("cannot parse result file: {}", output)
            ))? + start;
        Ok(output[start..end].parse()?)
    }

    pub fn id(&self) -> usize {
        self.connection_info.id
    }

    pub fn ip(&self) -> &str {
        &self.connection_info.ip
    }

    fn jar_path(&self) -> String {
        format!("{}/benchmarker/client.jar",
            &self.config.system.remote_work_dir
        )
    }

    fn log_path(&self) -> String {
        format!("{}/client-{}.log",
            &self.config.system.remote_work_dir,
            self.connection_info.id
        )
    }

    fn result_path(&self) -> String {
        format!("{}/results",
            &self.config.system.remote_work_dir
        )
    }

    fn grep_log(&self, keyword: &str) -> Result<String> {
        let cmd = format!("grep '{}' {}",
            keyword,
            self.log_path()
        );
        let output = command::ssh(
            &self.config.system.user_name,
            &self.connection_info.ip,
            &cmd
        )?;
        Ok(output)
    }
}