use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::BenchError;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub system: System,
    pub path: Path,
    pub machines: Machines
}

#[derive(Deserialize, Debug)]
pub struct System {
    pub user_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Path {
    pub remote_work_dir: String,
    pub jdk_dir: String,
    pub local_jdk_path: String,
    #[serde(skip)]
    pub jdk_package: String
}

#[derive(Deserialize, Debug)]
pub struct Machines {
    #[serde(skip)]
    pub all: Vec<String>,
    pub sequencers: Vec<String>,
    pub servers: Vec<String>,
    pub clients: Vec<String>
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, BenchError> {
        // Read the file
        let mut config_file = File::open(&path)
            .map_err(|e| BenchError::throw("cannot open config file", e))?;
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str)
            .map_err(|e| BenchError::throw("cannot read config file", e))?;
        let mut config: Config = toml::from_str(&config_str)
            .map_err(|e| BenchError::throw("cannot parse config file", e))?;

        // All ips
        config.generate_all_ips();

        // Get JDK package name
        let path = PathBuf::from(&config.path.local_jdk_path);
        match path.file_name() {
            Some(f) => {
                config.path.jdk_package = String::from(f.to_str().unwrap());
            },
            None => {
                return Err(BenchError::message("cannot recognize JDK's package"));
            }
        }

        Ok(config)
    }

    fn generate_all_ips(&mut self) {
        for ip in &self.machines.sequencers {
            self.machines.all.push(String::from(ip.as_str()));
        }
        for ip in &self.machines.servers {
            self.machines.all.push(String::from(ip.as_str()));
        }
        for ip in &self.machines.clients {
            self.machines.all.push(String::from(ip.as_str()));
        }
    }
}