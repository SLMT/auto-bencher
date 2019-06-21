
use log::*;

use crate::config::Config;
use crate::error::{Result, BenchError};
use crate::command;

pub struct Server {
    config: Config,
    address: String,
    bench_type: String,
    vm_args: String
}

impl Server {
    pub fn new(config: Config, address: String,
        bench_type: String, vm_args: String) -> Server {
        Server {
            config,
            address,
            bench_type,
            vm_args
        }
    }

    pub fn kill_existing_process(&self) -> Result<()> {
        debug!("Kill existing processes on the server...");
        let result = command::ssh(
            &self.config.system.user_name,
            &self.address,
            "pkill -f benchmarker"
        );
        match result {
            Err(BenchError::CommandFailedOnRemote(_, _, 1, _)) =>
                    info!("No existing process is found on '{}'", self.address),
            Err(e) => return Err(e),
            _ => {}
        }
        Ok(())
    }

    pub fn send_bench_dir(&self) -> Result<()> {
        debug!("Sending benchmarker to the server...");
        command::scp_to(
            true,
            &self.config.system.user_name,
            &self.address,
            "benchmarker",
            &self.config.system.remote_work_dir
        )?;
        Ok(())
    }

    pub fn delete_backup_db_dir(&self) -> Result<()> {
        debug!("Deleting backup dir on the server...");
        let cmd = format!("rm -rf {}",
            self.backup_db_path());
        let result = command::ssh(
            &self.config.system.user_name,
            &self.address,
            &cmd
        );
        match result {
            Err(BenchError::CommandFailedOnRemote(_, _, 1, _)) =>
                    debug!("No backup database is found on '{}'", self.address),
            Err(e) => return Err(e),
            _ => {}
        }
        Ok(())
    }

    pub fn backup_db(&self) -> Result<()> {
        debug!("Backing the db of the server...");
        let cmd = format!("cp -r {} {}",
            self.db_path(),
            self.backup_db_path()
        );
        command::ssh(
            &self.config.system.user_name,
            &self.address,
            &cmd
        );
        Ok(())
    }

    pub fn reset_db_dir(&self) -> Result<()> {
        debug!("Resetting the db of the server...");
        // delete the old db dir
        let cmd = format!("rm -rf {}",
            self.db_path());
        command::ssh(
            &self.config.system.user_name,
            &self.address,
            &cmd
        )?;
        // copy the backup for replacement
        let cmd = format!("cp -r {} {}",
            self.backup_db_path(),
            self.db_path()
        );
        command::ssh(
            &self.config.system.user_name,
            &self.address,
            &cmd
        )?;
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        info!("Starting the server...");
        // [db name]
        let prog_args = format!("{}", self.bench_type);
        let cmd = format!("{} {} -jar {} {} > {} 2>&1 &",
            self.config.jdk.remote_java_bin,
            self.vm_args,
            self.jar_path(),
            prog_args,
            self.log_path()
        );
        command::ssh(
            &self.config.system.user_name,
            &self.address,
            &cmd
        )?;
        Ok(())
    }

    pub fn check_for_ready(&self) -> Result<bool> {
        self.check_for_error()?;

        match self.grep_log("VanillaBench server ready") {
            Ok(_) => Ok(true),
            Err(BenchError::CommandFailedOnRemote(_, _, 1, _)) =>
                Ok(false),
            Err(e) => Err(e)
        }
    }

    pub fn check_for_error(&self) -> Result<()> {
        if let Ok(output) = self.grep_log("Exception") {
            return Err(BenchError::Message(
                format!("Server error: {}", output)));
        }

        if let Ok(output) = self.grep_log("error") {
            return Err(BenchError::Message(
                format!("Server error: {}", output)));
        }

        Ok(())
    }

    pub fn pull_log(&self) -> Result<()> {
        unimplemented!();
    }

    fn db_path(&self) -> String {
        format!("{}/databases/{}",
            &self.config.system.remote_work_dir,
            &self.bench_type
        )
    }

    fn backup_db_path(&self) -> String {
        format!("{}/databases/{}-backup",
            &self.config.system.remote_work_dir,
            &self.bench_type
        )
    }

    fn jar_path(&self) -> String {
        format!("{}/benchmarker/server.jar",
            &self.config.system.remote_work_dir
        )
    }

    fn log_path(&self) -> String {
        format!("{}/server.log",
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
            &self.address,
            &cmd
        )?;
        Ok(output)
    }
}