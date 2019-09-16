
pub mod init_env;
pub mod load;
pub mod benchmark;

use std::sync::{Arc, Barrier, RwLock};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use log::*;

use crate::error::{Result, BenchError};
use crate::parameters::Parameter;
use crate::config::Config;
use crate::command;
use crate::connections::{Server, Client, Action, ConnectionInfo};

const CHECKING_INTERVAL: u64 = 1;

enum ThreadResult {
    ServerSucceed,
    ClientSucceed(Option<u32>),
    Failed
}

fn run_server_and_client(config: &Config, parameter: &Parameter,
        db_name: &str, action: Action) -> Result<Vec<Option<u32>>> {
    
    // Generate connection information (ip, port)
    let (sequencer, server_list, client_list) =
        generate_connection_list(config, parameter)?;
    
    // Prepare the bench dir
    let vm_args = crate::preparation::prepare_bench_dir(
        &config, parameter, &sequencer, &server_list, &client_list)?;

    info!("Connecting to machines...");

    info!("Killing existing benchmarker processes...");
    kill_benchmarker_on_all_machines(config)?;

    // Use a mspc channel to collect results
    let (tx, rx): (Sender<ThreadResult>, Receiver<ThreadResult>)
        = mpsc::channel();
    let mut threads = Vec::new();
    let thread_count = match sequencer {
        Some(_) => server_list.len() + client_list.len() + 1,
        None => server_list.len() + client_list.len()
    };
    let barrier = Arc::new(Barrier::new(thread_count));

    // Create server connections
    let stop_sign = Arc::new(RwLock::new(false));
    for server_conn in &server_list {
        let handle = create_server_connection(
            barrier.clone(),
            stop_sign.clone(),
            config.clone(),
            server_conn.clone(),
            db_name.to_owned(), vm_args.clone(),
            false,
            tx.clone(),
            action
        );
        threads.push(handle);
    }

    // Create sequencer connection
    if let Some(seq_conn) = sequencer {
        let handle = create_server_connection(
            barrier.clone(),
            stop_sign.clone(),
            config.clone(),
            seq_conn.clone(),
            db_name.to_owned(), vm_args.clone(),
            true,
            tx.clone(),
            action
        );
        threads.push(handle);
    }

    // Create a client connection
    for client_conn in &client_list {
        let handle = create_client_connection(
            barrier.clone(),
            config.clone(),
            client_conn.clone(),
            vm_args.clone(),
            tx.clone(),
            action
        );
        threads.push(handle);
    }

    // Check if there is any error
    let mut client_results: Vec<Option<u32>> = Vec::new();
    for _ in 0 .. threads.len() {
        match rx.recv().unwrap() {
            ThreadResult::ClientSucceed(th) => {
                client_results.push(th);
                if client_results.len() >= client_list.len() {
                    // Notify the servers to finish
                    let mut stop = stop_sign.write().unwrap();
                    *stop = true;
                }
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

    Ok(client_results)
}

fn generate_connection_list(config: &Config, parameter: &Parameter)
    -> Result<(Option<ConnectionInfo>, Vec<ConnectionInfo>, Vec<ConnectionInfo>)> {
    
    let server_count: usize = parameter
        .get_autobencher_param("server_count")?.parse()?;
    let server_client_ratio: f64 = parameter
        .get_autobencher_param("server_client_ratio")?.parse()?;
    let max_server_per_machine: usize = parameter
        .get_autobencher_param("max_server_per_machine")?.parse()?;
    let max_client_per_machine: usize = parameter
        .get_autobencher_param("max_client_per_machine")?.parse()?;
    
    let client_count = (server_count as f64 * server_client_ratio) as usize;

    let sequencer = config.machines.sequencer.clone().map(|seq_ip| ConnectionInfo {
        id: server_count,
        ip: seq_ip,
        port: 30000
    });
    let server_list = ConnectionInfo::generate_connection_list(
        &config.machines.servers,
        server_count,
        max_server_per_machine
    )?;
    let client_list = ConnectionInfo::generate_connection_list(
        &config.machines.clients,
        client_count,
        max_client_per_machine
    )?;

    Ok((sequencer, server_list, client_list))
}

fn kill_benchmarker_on_all_machines(config: &Config) -> Result<()> {
    for machine in &config.machines.all {
        let result = command::ssh(
            &config.system.user_name,
            &machine,
            "pkill -f benchmarker"
        );
        match result {
            Err(BenchError::CommandFailedOnRemote(_, _, 1, _)) =>
                    info!("No existing process is found on '{}'", machine),
            Err(e) => return Err(e),
            _ => {}
        }
    }
    Ok(())
}

fn create_server_connection(barrier: Arc<Barrier>, stop_sign: Arc<RwLock<bool>>,
        config: Config, conn_info: ConnectionInfo, db_name: String,
        vm_args: String, is_sequencer: bool,
        result_ch: Sender<ThreadResult>, action: Action)
        -> JoinHandle<()> {
    thread::spawn(move || {
        let server = Server::new(config, conn_info,
            db_name, vm_args, is_sequencer);
        let result = match execute_server_thread(&server, barrier,
                stop_sign, action) {
            Err(e) => {
                error!("Server {} (on {}) occurs an error: {}",
                    server.id(), server.ip(), e);
                ThreadResult::Failed
            },
            _ => ThreadResult::ServerSucceed
        };
        info!("The server finished.");
        result_ch.send(result).unwrap();
    })
}

fn execute_server_thread(server: &Server, barrier: Arc<Barrier>,
    stop_sign: Arc<RwLock<bool>>, action: Action) -> Result<()> {
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

    info!("Server {} is ready.", server.id());

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
        config: Config, conn_info: ConnectionInfo,
        vm_args: String, result_ch: Sender<ThreadResult>,
        action: Action) -> JoinHandle<()> {
    thread::spawn(move || {
        let client = Client::new(config, conn_info, vm_args);
        let result = match execute_client_thread(&client, barrier, action) {
            Err(e) => {
                error!("Client {} (on {}) occurs an error: {}",
                    client.id(), client.ip(), e);
                ThreadResult::Failed
            },
            Ok(th) => ThreadResult::ClientSucceed(th)
        };
        info!("Client {} finished.", client.id());
        result_ch.send(result).unwrap();
    })
}

fn execute_client_thread(client: &Client, barrier: Arc<Barrier>,
        action: Action) -> Result<Option<u32>> {
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
        info!("The total throughput of client {} is {}",
            client.id(), throughput);
        Ok(Some(throughput))
    } else {
        Ok(None)
    }
}
