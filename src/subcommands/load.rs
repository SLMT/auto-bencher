
use std::path::PathBuf;
use std::sync::{Arc, Barrier, RwLock};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use colored::*;
use log::*;
use clap::{ArgMatches, Arg, App, SubCommand};

use crate::error::{Result, BenchError};
use crate::config::Config;
use crate::parameters::ParameterList;
use crate::connections::{Server, Client, Action};

const CHECKING_INTERVAL: u64 = 3;

enum ThreadResult {
    ServerSucceed,
    ClientSucceed,
    Failed
}

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("load")
                .arg(Arg::with_name("BENCH TYPE")
                    .help("Sets the type of the benchmark")
                    .required(true)
                    .index(1))
                .about("loads the testbed of the specified benchmark for the given number of machines")
}

pub fn execute(config: &Config, args: &ArgMatches) -> Result<()> {
    let bench_type = args.value_of("BENCH TYPE").unwrap();
    
    info!("Preparing for loading {} benchmarks...",
        bench_type.cyan());

    // Read the parameter file
    let param_list = read_parameter_file(bench_type)?;

    // The file should only produce single "Parameter"
    let param_list = param_list.to_vec();
    if param_list.len() > 1 {
        return Err(BenchError::Message(format!(
            "{}'s parameter file contains more than one combination", 
            bench_type.cyan()
        )))
    }

    // Prepare the bench dir
    let vm_args = super::prepare_bench_dir(&config, &param_list[0])?;

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
        tx.clone()
    );
    threads.push(handle);

    // Create a client connection
    let handle = create_client_connection(
        barrier.clone(),
        config.clone(),
        config.machines.client.clone(),
        vm_args.clone(),
        tx.clone()
    );
    threads.push(handle);

    // Check if there is any error
    for _ in 0 .. threads.len() {
        match rx.recv().unwrap() {
            ThreadResult::ClientSucceed => {
                // Notify the servers to finish
                let mut stop = stop_sign.write().unwrap();
                *stop = true;
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

    // Show the final result (where is the database, the size...)
    info!("Loading of benchmark {} finished.", bench_type);

    Ok(())
}

fn read_parameter_file(bench_type: &str) -> Result<ParameterList> {
    let mut param_file = PathBuf::new();
    param_file.push("parameters");
    param_file.push("loading");
    param_file.push(bench_type);
    param_file.set_extension("toml");

    info!("Reading the parameter file: '{}'", param_file.to_str().unwrap());

    ParameterList::from_file(&param_file)
}

fn create_server_connection(barrier: Arc<Barrier>, stop_sign: Arc<RwLock<bool>>,
        config: Config, ip: String, bench_type: String,
        vm_args: String, result_ch: Sender<ThreadResult>) -> JoinHandle<()> {
    thread::spawn(move || {
        let server = Server::new(config, ip.clone(),
            bench_type, vm_args);
        let result = match execute_server_thread(server, barrier, stop_sign) {
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
    stop_sign: Arc<RwLock<bool>>) -> Result<()> {
    server.kill_existing_process()?;
    server.send_bench_dir()?;
    server.delete_backup_db_dir()?;

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

    server.backup_db()?;

    Ok(())
}

fn create_client_connection(barrier: Arc<Barrier>,
        config: Config, ip: String, vm_args: String,
        result_ch: Sender<ThreadResult>) -> JoinHandle<()> {
    thread::spawn(move || {
        let client = Client::new(config, ip.clone(), vm_args);
        let result = match execute_client_thread(client, barrier) {
            Err(e) => {
                error!("The client ({}) occurs an error: {}",
                    ip, e);
                ThreadResult::Failed
            },
            _ => ThreadResult::ClientSucceed
        };
        info!("The client finished.");
        result_ch.send(result).unwrap();
    })
}

fn execute_client_thread(client: Client, barrier: Arc<Barrier>) -> Result<()> {
    client.kill_existing_process()?;
    client.clean_previous_results()?;
    client.send_bench_dir()?;

    // Wait for all servers ready
    barrier.wait(); // prepared
    barrier.wait(); // ready

    client.start(Action::Loading)?;
    while client.check_for_finished()? {
        thread::sleep(Duration::from_secs(CHECKING_INTERVAL));
    }

    Ok(())
}