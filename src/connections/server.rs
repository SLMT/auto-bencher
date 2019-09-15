
use log::*;

use crate::config::Config;
use crate::error::{Result, BenchError};
use crate::command;
use super::ConnectionInfo;

pub struct Server {
    config: Config,
    connection_info: ConnectionInfo,
    db_name: String,
    vm_args: String,
    is_sequencer: bool
}

impl Server {
    pub fn new(config: Config, connection_info: ConnectionInfo,
        db_name: String, vm_args: String,
        is_sequencer: bool) -> Server {
        
        let db_name = if is_sequencer {
            format!("{}-seq", db_name)
        } else {
            format!("{}-{}", db_name, connection_info.id)
        };

        Server {
            config,
            connection_info,
            db_name,
            vm_args,
            is_sequencer
        }
    }

    pub fn send_bench_dir(&self) -> Result<()> {
        debug!("Sending benchmarker to server {}...", self.id());
        command::scp_to(
            true,
            &self.config.system.user_name,
            &self.connection_info.ip,
            "benchmarker",
            &self.config.system.remote_work_dir
        )?;
        Ok(())
    }

    pub fn delete_db_dir(&self) -> Result<()> {
        let cmd = format!("rm -rf {}",
            self.db_path());
        let result = command::ssh(
            &self.config.system.user_name,
            &self.connection_info.ip,
            &cmd
        );
        match result {
            Err(BenchError::CommandFailedOnRemote(_, _, 1, _)) =>
                    debug!("No backup database is found on '{}'", self.connection_info.ip),
            Err(e) => return Err(e),
            _ => {}
        }
        Ok(())
    }

    pub fn delete_backup_db_dir(&self) -> Result<()> {
        debug!("Deleting backup dir on server {}...", self.id());
        let cmd = format!("rm -rf {}",
            self.backup_db_path());
        let result = command::ssh(
            &self.config.system.user_name,
            &self.connection_info.ip,
            &cmd
        );
        match result {
            Err(BenchError::CommandFailedOnRemote(_, _, 1, _)) =>
                    debug!("No backup database is found on '{}'", self.connection_info.ip),
            Err(e) => return Err(e),
            _ => {}
        }
        Ok(())
    }

    pub fn backup_db(&self) -> Result<()> {
        debug!("Backing the db of server {}...", self.id());
        let cmd = format!("cp -r {} {}",
            self.db_path(),
            self.backup_db_path()
        );
        command::ssh(
            &self.config.system.user_name,
            &self.connection_info.ip,
            &cmd
        )?;
        Ok(())
    }

    pub fn reset_db_dir(&self) -> Result<()> {
        debug!("Resetting the db of server {}...", self.id());
        self.delete_db_dir()?;
        // copy the backup for replacement
        let cmd = format!("cp -r {} {}",
            self.backup_db_path(),
            self.db_path()
        );
        command::ssh(
            &self.config.system.user_name,
            &self.connection_info.ip,
            &cmd
        )?;
        Ok(())
    }

    pub fn start(&self) -> Result<()> {
        info!("Starting server {}...", self.id());
        // [db name] [server id] ([is sequencer])
        let prog_args = if self.is_sequencer {
            format!("{} {}", self.db_name, self.connection_info.id)
        } else {
            format!("{} {} 1", self.db_name, self.connection_info.id)
        };
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

    pub fn id(&self) -> usize {
        self.connection_info.id
    }

    pub fn ip(&self) -> &str {
        &self.connection_info.ip
    }

    fn db_path(&self) -> String {
        format!("{}/databases/{}",
            &self.config.system.remote_work_dir,
            &self.db_name
        )
    }

    fn backup_db_path(&self) -> String {
        format!("{}/databases/{}-backup",
            &self.config.system.remote_work_dir,
            &self.db_name
        )
    }

    fn jar_path(&self) -> String {
        format!("{}/benchmarker/server.jar",
            &self.config.system.remote_work_dir
        )
    }

    fn log_path(&self) -> String {
        if self.is_sequencer {
            format!("{}/server-seq.log",
                &self.config.system.remote_work_dir
            )
        } else {
            format!("{}/server-{}.log",
                &self.config.system.remote_work_dir,
                &self.connection_info.id
            )
        }
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