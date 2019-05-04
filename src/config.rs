use std::fs::File;
use std::io::Read;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub system: System,
    pub machines: Machines
}

#[derive(Deserialize, Debug)]
pub struct System {
    pub user_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Machines {
    #[serde(skip)]
    pub alls: Vec<String>,
    pub sequencers: Vec<String>,
    pub servers: Vec<String>,
    pub clients: Vec<String>
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, String> {
        // Read the file
        let mut config_file = File::open(&path)
            .map_err(|e| format!("cannot open config file: {}", e.to_string()))?;
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str)
            .map_err(|e| format!("cannot read config file: {}", e.to_string()))?;
        let mut config: Config = toml::from_str(&config_str)
            .map_err(|e| format!("cannot parse config file: {}", e.to_string()))?;

        // All ips
        config.generate_all_ips();

        Ok(config)
    }

    fn generate_all_ips(&mut self) {
        for ip in &self.machines.sequencers {
            self.machines.alls.push(String::from(ip.as_str()));
        }
        for ip in &self.machines.servers {
            self.machines.alls.push(String::from(ip.as_str()));
        }
        for ip in &self.machines.clients {
            self.machines.alls.push(String::from(ip.as_str()));
        }
    }
}