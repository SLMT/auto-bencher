
use std::collections::HashMap;
use std::path::Path;
use std::fs::{self, File};
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
    fn from_file(id: &str, path: &Path) -> Result<PropertiesFile> {
        let file = File::open(path)?;
        let properties = java_properties::read(BufReader::new(file))?;
        let filename = path.file_stem().unwrap().to_str().unwrap().to_owned();

        Ok(PropertiesFile {
            id: id.to_owned(),
            filename,
            properties
        })
    }

    fn set(&mut self, property: &str, value: &str) {
        if let Some(val) = self.properties.get_mut(property) {
            *val = value.to_owned();
        }
    }

    fn output_to_file(&self, dir_path: &Path) -> Result<()> {
        let mut file_path = dir_path.join(&self.filename);
        file_path = file_path.with_extension("properties");
        let file = File::create(file_path)?;
        java_properties::write(file, &self.properties)?;
        Ok(())
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
            files.insert(file.filename.clone(), file);
        }

        Ok(PropertiesFileMap {
            files
        })
    }

    pub fn set(&mut self, filename: &str, property: &str, value: &str) {
        if let Some(file) = self.files.get_mut(filename) {
            file.set(property, value);
        }
    }

    pub fn output_to_dir(&self, dir_path: &Path) -> Result<()> {
        fs::create_dir_all(dir_path)?;
        for (_, file) in &self.files {
            file.output_to_file(dir_path)?;
        }
        Ok(())
    }

    pub fn get_vm_args(&self, prop_dir_path: &Path) -> Result<String> {
        let mut vm_args = String::new();

        for (_, file) in &self.files {
            vm_args.push_str("-D");
            vm_args.push_str(&file.id);
            vm_args.push_str("=");
            let file_path = prop_dir_path.join(&file.filename);
            vm_args.push_str(file_path.to_str().unwrap());
            vm_args.push_str(".properties ");
        }
        vm_args.remove(vm_args.len() - 1); // remove the last " "

        Ok(vm_args)
    }
}