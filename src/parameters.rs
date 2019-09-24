
use std::path::Path;

use toml::Value as TomlValue;

use crate::error::{Result, BenchError};
use crate::properties::PropertiesFileMap;

#[derive(Debug, Clone)]
pub struct Parameter<'a> {
    // (filename, (property, value))
    params: Vec<(&'a str, Vec<(&'a str, &'a str)>)>
}

impl<'a> Parameter<'a> {
    fn empty() -> Parameter<'a> {
        Parameter {
            params: Vec::new()
        }
    }

    // We assume that there will be a few files
    // and no single line will be added twice
    fn add_param(&mut self, filename: &'a str,
            property: &'a str, value: &'a str) {
        let mut found = false;

        for (param_file, param_lines) in self.params.iter_mut() {
            if *param_file == filename {
                param_lines.push((property, value));
                found = true;
                break;
            }
        }

        if !found {
            let param_lines = vec![(property, value)];
            self.params.push((filename, param_lines))
        }
    }

    pub fn get_autobencher_param(&self, key: &str) -> Result<&str> {
        for (param_file, param_lines) in &self.params {
            if *param_file == "auto_bencher" {
                for (prop, value) in param_lines {
                    if *prop == key {
                        return Ok(value)
                    }
                }
            }
        }

        // The parameter for the auto-bencher must exist
        Err(BenchError::Message(format!(
            "Cannot find parameter \"{}\" for the auto-bencher",
            key
        )))
    }

    pub fn override_properties(&self, files: &mut PropertiesFileMap) {
        for (param_file, param_lines) in &self.params {
            if *param_file == "auto_bencher" {
                continue;
            }

            for (prop, value) in param_lines {
                files.set(param_file, prop, value);
            }
        }
    }
    
    pub fn get_properties(&self) -> Vec<&'a str> {
        let mut properties = Vec::new();
        for (_, param_lines) in &self.params {
            for (prop, _) in param_lines {
                properties.push(*prop);
            }
        }
        properties
    }
    
    pub fn get_properties_values(&self) -> Vec<&'a str> {
        let mut values = Vec::new();
        for (_, param_lines) in &self.params {
            for (_, value) in param_lines {
                values.push(*value);
            }
        }
        values
    }
}

#[derive(Debug)]
pub struct ParameterList {
    // (filename, (property, value list))
    param_lists: Vec<(String, Vec<(String, String)>)>
}

impl ParameterList {
    pub fn from_file(file_path: &Path) -> Result<ParameterList> {
        // Read the parameter file
        let toml_str = std::fs::read_to_string(file_path)?;
        let parameter_list: TomlValue = toml_str.parse()?;

        // Read each parameter
        let mut param_lists = Vec::new();
        if let TomlValue::Table(files) = parameter_list {
            for (filename, toml_table) in files {
                if let TomlValue::Table(map) = toml_table {
                    let params = map.into_iter()
                        .map(|(k, v)| (k, v.as_str().unwrap().to_owned()))
                        .collect();
                    param_lists.push((filename.clone(), params));
                }
            }
        }

        Ok(ParameterList {
            param_lists
        })
    }

    pub fn to_vec(&self) -> Vec<Parameter> {
        let mut result = Vec::new();
        self.iterate_parameters(0, 0, Parameter::empty(), &mut result);
        result
    }

    // Find all combinations of parameters. Implemented by recursion
    fn iterate_parameters<'a>(&'a self, file_id: usize, line_id: usize,
            current: Parameter<'a>, results: &mut Vec<Parameter<'a>>) {
        // Check if the file id exceeds
        if file_id < self.param_lists.len() {
            let (filename, param_lines) = &self.param_lists[file_id];

            // Check if the line id exceeds
            if line_id < param_lines.len() {
                // Read a line and split it
                let (prop, value_list) = &param_lines[line_id];
                for value in value_list.split(" ") {
                    let mut new = current.clone();
                    new.add_param(filename.as_str(), prop.as_str(), value);
                    self.iterate_parameters(file_id, line_id + 1, new, results);
                }
            } else {
                // To next file
                self.iterate_parameters(file_id + 1, 0, current, results);
            }
        } else {
            // Reach the bottom, save the result
            results.push(current);
        }
    }
}