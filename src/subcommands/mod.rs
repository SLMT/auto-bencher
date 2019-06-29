
pub mod init_env;
pub mod load;
pub mod benchmark;

use std::fs;
use std::sync::{Arc, Barrier, RwLock};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::path::{Path, PathBuf};

use log::*;

use crate::error::{Result, BenchError};
use crate::parameters::Parameter;
use crate::properties::PropertiesFileMap;
use crate::command;
use crate::config::Config;
use crate::connections::{Server, Client, Action};

const BENCH_DIR: &'static str = "benchmarker";
const PROP_DIR: &'static str = "props";
const CHECKING_INTERVAL: u64 = 1;

enum ThreadResult {
    ServerSucceed,
    ClientSucceed(Option<u32>),
    Failed
}

fn run_server_and_client(config: &Config, parameter: &Parameter,
        bench_type: &str, action: Action) -> Result<Option<u32>> {
    // Prepare the bench dir
    let vm_args = prepare_bench_dir(&config, parameter)?;

    info!("Connecting to machines...");

    // Use a mspc channel to collect results
    let (tx, rx): (Sender<ThreadResult>, Receiver<ThreadResult>)
        = mpsc::channel();
    let barrier = Arc::new(Barrier::new(2));
    let mut threads = Vec::new();

    // Create server connections
    let stop_sign = Arc::new(RwLock::new(false));
    let handle = create_server_connection(
        barrier.clone(),
        stop_sign.clone(),
        config.clone(),
        config.machines.server.clone(),
        bench_type.to_owned(), vm_args.clone(),
        tx.clone(),
        action
    );
    threads.push(handle);

    // Create a client connection
    let handle = create_client_connection(
        barrier.clone(),
        config.clone(),
        config.machines.client.clone(),
        vm_args.clone(),
        tx.clone(),
        action
    );
    threads.push(handle);

    // Check if there is any error
    let mut throughput: Option<u32> = None;
    for _ in 0 .. threads.len() {
        match rx.recv().unwrap() {
            ThreadResult::ClientSucceed(th) => {
                // Notify the servers to finish
                let mut stop = stop_sign.write().unwrap();
                *stop = true;
                throughput = th;
            },
            ThreadResult::Failed => {
                return Err(BenchError::Message(
                    "A thread exits with an error".to_owned()
                ));
            },
            _ => {}
        }
    }

    // Wait for the threads finish
    for thread in threads {
        thread.join().unwrap();
    }

    Ok(throughput)
}

fn create_server_connection(barrier: Arc<Barrier>, stop_sign: Arc<RwLock<bool>>,
        config: Config, ip: String, bench_type: String,
        vm_args: String, result_ch: Sender<ThreadResult>,
        action: Action) -> JoinHandle<()> {
    thread::spawn(move || {
        let server = Server::new(config, ip.clone(),
            bench_type, vm_args);
        let result = match execute_server_thread(server, barrier,
                stop_sign, action) {
            Err(e) => {
                error!("The server ({}) occurs an error: {}", ip, e);
                ThreadResult::Failed
            },
            _ => ThreadResult::ServerSucceed
        };
        info!("The server finished.");
        result_ch.send(result).unwrap();
    })
}

fn execute_server_thread(server: Server, barrier: Arc<Barrier>,
    stop_sign: Arc<RwLock<bool>>, action: Action) -> Result<()> {
    server.kill_existing_process()?;
    server.send_bench_dir()?;

    match action {
        Action::Loading => {
            server.delete_db_dir()?;
            server.delete_backup_db_dir()?;
        },
        Action::Benchmarking => {
            server.reset_db_dir()?;
        }
    }

    // Wait for other servers prepared
    barrier.wait();

    server.start()?;
    while !server.check_for_ready()? {
        thread::sleep(Duration::from_secs(CHECKING_INTERVAL));
    }

    info!("The server is ready.");

    // Wait for all servers ready
    barrier.wait();

    let mut stop = false;
    while !stop {
        server.check_for_error()?;
        thread::sleep(Duration::from_secs(CHECKING_INTERVAL));
        stop = *(stop_sign.read()?);
    }

    if let Action::Loading = action {
        server.backup_db()?;
    }

    Ok(())
}

fn create_client_connection(barrier: Arc<Barrier>,
        config: Config, ip: String, vm_args: String,
        result_ch: Sender<ThreadResult>,
        action: Action) -> JoinHandle<()> {
    thread::spawn(move || {
        let client = Client::new(config, ip.clone(), vm_args);
        let result = match execute_client_thread(client, barrier, action) {
            Err(e) => {
                error!("The client ({}) occurs an error: {}",
                    ip, e);
                ThreadResult::Failed
            },
            Ok(th) => ThreadResult::ClientSucceed(th)
        };
        info!("The client finished.");
        result_ch.send(result).unwrap();
    })
}

fn execute_client_thread(client: Client, barrier: Arc<Barrier>,
        action: Action) -> Result<Option<u32>> {
    client.kill_existing_process()?;
    client.clean_previous_results()?;
    client.send_bench_dir()?;

    // Wait for the server ready
    barrier.wait(); // prepared
    barrier.wait(); // ready

    client.start(action)?;
    while !client.check_for_finished(action)? {
        thread::sleep(Duration::from_secs(CHECKING_INTERVAL));
    }

    if let Action::Benchmarking = action {
        // client.pull_csv()?;
        let throughput = client.get_total_throughput()?;
        info!("The total throughput is {}", throughput);
        Ok(Some(throughput))
    } else {
        Ok(None)
    }
}

// Output: vm args for properties files
fn prepare_bench_dir(config: &Config, parameter: &Parameter) -> Result<String> {
    info!("Preparing the benchmarker directory...");

    // Ensure the existance of the benchmarker dir
    fs::create_dir_all(BENCH_DIR)?;

    // Copy the jar files to the benchmark dir
    if let Some(dirname) = parameter.get_basic_param("JAR_DIR") {
        copy_jars(dirname)?;
    } else {
        return Err(BenchError::Message(
            "No \"JAR_DIR\" is provided in the parameter file".to_owned()
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