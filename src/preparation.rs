
use std::fs;
use std::path::{Path, PathBuf};

use log::*;

use crate::error::Result;
use crate::parameters::Parameter;
use crate::properties::PropertiesFileMap;
use crate::command;
use crate::config::Config;
use crate::connections::ConnectionInfo;

const BENCH_DIR: &'static str = "benchmarker";
const PROP_DIR: &'static str = "props";

// Output: vm args for properties files
pub fn prepare_bench_dir(config: &Config, parameter: &Parameter,
        sequencer: &Option<ConnectionInfo>, server_list: &Vec<ConnectionInfo>,
        client_list: &Vec<ConnectionInfo>) -> Result<String> {
    info!("Preparing the benchmarker directory...");

    // Ensure the existance of the benchmarker dir
    fs::create_dir_all(BENCH_DIR)?;

    // Copy the jar files to the benchmark dir
    let dirname = parameter.get_autobencher_param("jar_dir")?;
    copy_jars(dirname)?;

    // Read the default properties
    let mut map = PropertiesFileMap::from_dir(&Path::new("properties"))?;

    // Apply the parameters
    parameter.override_properties(&mut map);
    set_paths(config, &mut map);
    set_connection_properties(&mut map, sequencer, server_list, client_list)?;
    set_elasql_properties(&mut map, server_list.len());

    // Generate the properties files to the benchmark dir
    let prop_dir_path: PathBuf = [BENCH_DIR, PROP_DIR].iter().collect();
    map.output_to_dir(&prop_dir_path)?;

    let mut remote_prop_dir_path = PathBuf::new();
    remote_prop_dir_path.push(&config.system.remote_work_dir);
    remote_prop_dir_path.push(prop_dir_path);
    map.get_vm_args(&remote_prop_dir_path)
}

fn copy_jars(dir_name: &str) -> Result<()> {
    let dir_path = format!("jars/{}", dir_name);
    let filenames = vec!["server.jar", "client.jar"];
    for filename in filenames {
        let jar_path = format!("{}/{}", dir_path, filename);
        // Check if the jar exists
        command::ls(&jar_path)?;
        // Copy it to the benchmarker directory
        command::cp(false, &jar_path, BENCH_DIR)?;
    }
    Ok(())
}

fn set_paths(config: &Config, map: &mut PropertiesFileMap) {
    map.set(
        "vanilladb",
        "org.vanilladb.core.storage.file.FileMgr.DB_FILES_DIR",
        &format!("{}/databases", config.system.remote_work_dir)
    );
    map.set(
        "vanillabench",
        "org.vanilladb.bench.StatisticMgr.OUTPUT_DIR",
        &format!("{}/results", config.system.remote_work_dir)
    );
}

fn set_connection_properties(map: &mut PropertiesFileMap,
        sequencer: &Option<ConnectionInfo>, server_list: &Vec<ConnectionInfo>,
        client_list: &Vec<ConnectionInfo>) -> Result<()> {
    
    // Set server view
    let mut server_view = String::new();
    for server in server_list {
        if server.id > 0 {
            server_view.push_str(", ");
        }
        server_view.push_str(&server.to_string());
    }
    if let Some(seq_info) = sequencer {
        server_view.push_str(", ");
        server_view.push_str(&seq_info.to_string());
    }
    map.set(
        "vanillacomm",
        "org.vanilladb.comm.view.ProcessView.SERVER_VIEW",
        &server_view
    );

    // Set client view
    let mut client_view = String::new();
    for client in client_list {
        if client.id > 0 {
            client_view.push_str(", ");
        }
        client_view.push_str(&client.to_string());
    }
    map.set(
        "vanillacomm",
        "org.vanilladb.comm.view.ProcessView.CLIENT_VIEW",
        &client_view
    );

    // Set stand alone sequencer
    if let Some(_) = sequencer {
        map.set(
            "vanillacomm",
            "org.vanilladb.comm.ProcessView.STAND_ALONE_SEQUENCER",
            "true"
        );
    } else {
        map.set(
            "vanillacomm",
            "org.vanilladb.comm.ProcessView.STAND_ALONE_SEQUENCER",
            "false"
        );
    }

    Ok(())
}

fn set_elasql_properties(map: &mut PropertiesFileMap, server_count: usize) {
    map.set(
        "elasql",
        "org.elasql.storage.metadata.PartitionMetaMgr.NUM_PARTITIONS",
        &server_count.to_string()
    );
}