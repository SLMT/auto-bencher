
pub mod init_env;
pub mod load;

use std::path::{Path, PathBuf};
use std::fs;

use log::*;

use crate::error::{Result, BenchError};
use crate::parameters::Parameter;
use crate::properties::PropertiesFileMap;
use crate::command;
use crate::config::Config;

const BENCH_DIR: &'static str = "benchmarker";
const PROP_DIR: &'static str = "props";

// Output: vm args for properties files
fn prepare_bench_dir(config: &Config, parameter: &Parameter) -> Result<String> {
    info!("Preparing the benchmarker directory...");

    // Ensure the existance of the benchmarker dir
    fs::create_dir_all(BENCH_DIR)?;

    // Copy the jar files to the benchmark dir
    if let Some(dirname) = parameter.get_basic_param("jar.dir") {
        copy_jars(dirname)?;
    } else {
        return Err(BenchError::Message(
            "No \"jar.dir\" is provided in the parameter file".to_owned()
        ))
    }

    // Read the default properties
    let mut map = PropertiesFileMap::from_dir(&Path::new("properties"))?;

    // Apply the parameters
    parameter.override_properties(&mut map);
    set_paths(config, &mut map);
    map.set(
        "vanillabench",
        "org.vanilladb.bench.BenchmarkerParameters.SERVER_IP",
        &config.machines.server
    );

    // Generate the properties files to the benchmark dir
    let prop_dir_path: PathBuf = [BENCH_DIR, PROP_DIR].iter().collect();
    map.output_to_dir(&prop_dir_path)?;

    let mut remote_prop_dir_path = PathBuf::new();
    remote_prop_dir_path.push(&config.system.remote_work_dir);
    remote_prop_dir_path.push(prop_dir_path);
    map.get_vm_args(&remote_prop_dir_path)
}

fn copy_jars(dir_name: &str) -> Result<()> {
    let dir_path: PathBuf = ["jars", dir_name].iter().collect();

    let filenames = vec!["server.jar", "client.jar"];
    for filename in filenames {
        let jar_path = dir_path.join(filename);
        // Check if the jar exists
        command::ls(jar_path.to_str().unwrap())?;
        // Copy it to the benchmarker directory
        command::cp(false, jar_path.to_str().unwrap(), BENCH_DIR)?;
    }

    Ok(())
}

fn set_paths(config: &Config, map: &mut PropertiesFileMap) {
    let db_path: PathBuf = [
        &config.system.remote_work_dir,
        "databases"
    ].iter().collect();
    map.set(
        "vanilladb",
        "org.vanilladb.core.storage.file.FileMgr.DB_FILES_DIR",
        db_path.to_str().unwrap()
    );
    let result_path: PathBuf = [
        &config.system.remote_work_dir,
        "results"
    ].iter().collect();
    map.set(
        "vanillabench",
        "org.vanilladb.bench.StatisticMgr.OUTPUT_DIR",
        result_path.to_str().unwrap()
    );
}