use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::string::ToString;

use serde::Deserialize;

use crate::error::{Result, BenchError};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub system: System,
    pub jdk: Jdk,
    pub machines: Machines
}

#[derive(Deserialize, Debug, Clone)]
pub struct System {
    pub user_name: String,
    pub remote_work_dir: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Jdk {
    pub use_custom_jdk: bool,
    pub dir_name: String,
    pub package_path: String,
    pub jvm_args: String,
    #[serde(skip)]
    pub package_filename: String,
    #[serde(skip)]
    pub remote_java_bin: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Machines {
    #[serde(skip)]
    pub all: Vec<String>,
    pub sequencer: Option<String>,
    pub servers: Vec<String>,
    pub clients: Vec<String>
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config> {
        // Read the file
        let mut config_file = File::open(&path)?;
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str)?;
        let mut config: Config = toml::from_str(&config_str)?;

        // All ips
        config.generate_all_ips();

        // Get JDK package name
        let path = PathBuf::from(&config.jdk.package_path);
        match path.file_name() {
            Some(f) => {
                config.jdk.package_filename = f.to_str().unwrap().to_owned();
            },
            None => {
                return Err(BenchError::Message(
                    "cannot get the file name of the JDK".to_owned()))
            }
        }

        // Set the path to java
        let mut path = PathBuf::from(&config.system.remote_work_dir);
        path.push(&config.jdk.dir_name);
        path.push("bin");
        path.push("java");
        config.jdk.remote_java_bin = path.display().to_string();

        Ok(config)
    }

    fn generate_all_ips(&mut self) {
        if let Some(seq) = &self.machines.sequencer {
            self.machines.all.push(seq.clone());
        }
        self.machines.all.append(&mut self.machines.servers.clone());
        self.machines.all.append(&mut self.machines.clients.clone());
    }
}