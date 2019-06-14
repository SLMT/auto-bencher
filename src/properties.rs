
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;

use serde::Deserialize;

use crate::error::Result;

#[derive(Debug)]
struct PropertiesFile {
    id: String,
    filename: String, // without ".properties"
    properties: HashMap<String, String>
}

impl PropertiesFile {
    pub fn from_file(id: &str, path: &Path) -> Result<PropertiesFile> {
        let file = File::open(path)?;
        let properties = java_properties::read(BufReader::new(file))?;
        let filename = path.file_stem().unwrap().to_str().unwrap().to_owned();

        Ok(PropertiesFile {
            id: id.to_owned(),
            filename,
            properties
        })
    }
}

#[derive(Deserialize, Debug)]
struct Setting {
    id: String,
    filename: String
}

#[derive(Debug)]
pub struct PropertiesFileMap {
    // filename => PropertiesFile
    files: HashMap<String, PropertiesFile>
}

impl PropertiesFileMap {
    pub fn from_dir(input_dir: &Path) -> Result<PropertiesFileMap> {
        // Read the setting
        let settings_file = input_dir.join("settings.json");
        let json_str = std::fs::read_to_string(settings_file)?;
        let settings: Vec<Setting> = serde_json::from_str(&json_str)?;

        // Read each properties file
        let mut files: HashMap<String, PropertiesFile> = HashMap::new();
        for setting in settings {
            let path = input_dir.join(setting.filename);
            let file = PropertiesFile::from_file(
                    &setting.id, &path)?;
            files.insert(setting.id.clone(), file);
        }

        Ok(PropertiesFileMap {
            files
        })
    }
}