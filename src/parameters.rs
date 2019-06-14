
use std::io::Read;
use std::collections::HashMap;
use std::str::FromStr;
use std::path::Path;

use toml::Value as TomlValue;

use crate::error::Result;

// TODO: implement this
struct Parameter {
    params_map: HashMap<String, HashMap<String, String>>
}

// TODO: implement this
struct ParameterIter {

}

impl Iterator for ParameterIter {
    type Item = Parameter;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!();
    } 
}

#[derive(Debug)]
struct ParameterList {
    params_map: HashMap<String, HashMap<String, String>>
}

// TODO: implement display
impl ParameterList {
    pub fn from_file(file_path: &Path) -> Result<ParameterList> {
        // Read the parameter file
        let toml_str = std::fs::read_to_string(file_path)?;
        let parameter_list: TomlValue = toml_str.parse()?;

        // Read each parameter
        dbg!(parameter_list);

        unimplemented!();
    }
}
