
mod server;
mod client;

use std::sync::{Arc, Barrier, RwLock};
use std::sync::mpsc::{self, Sender, Receiver};

use log::*;

use crate::error::{Result, BenchError};
use crate::config::Config;
use crate::connections::{Action, ConnectionInfo};

const CHECKING_INTERVAL: u64 = 1;

pub enum ThreadResult {
    ServerSucceed,
    ClientSucceed(Option<u32>),
    Failed
}

pub fn run_in_threads(config: &Config, db_name: &str,
        action: Action, report_dir: Option<String>,
        vm_args: &str,
        sequencer: Option<ConnectionInfo>,
        server_list: Vec<ConnectionInfo>,
        client_list: Vec<ConnectionInfo>)
        -> Result<Vec<Option<u32>>> {
    // Use a mspc channel to collect results
    let (tx, rx): (Sender<ThreadResult>, Receiver<ThreadResult>)
        = mpsc::channel();
    let mut threads = Vec::new();
    
    // Calculate number of threads
    let thread_count = match sequencer {
        Some(_) => server_list.len() + client_list.len() + 1,
        None => server_list.len() + client_list.len()
    };
    let barrier = Arc::new(Barrier::new(thread_count));

    // Add other vm arguments for servers
    let mut server_vm_args = vm_args.to_owned();
    server_vm_args.push_str(" ");
    server_vm_args.push_str(&config.jdk.vmargs.servers);

    // Create server connections
    let stop_sign = Arc::new(RwLock::new(false));
    for server_conn in &server_list {
        let handle = server::create_server_thread(
            barrier.clone(),
            stop_sign.clone(),
            config.clone(),
            server_conn.clone(),
            db_name.to_owned(), server_vm_args.clone(),
            false,
            tx.clone(),
            action
        );
        threads.push(handle);
    }

    // Create sequencer connection
    if let Some(seq_conn) = sequencer {
        // Add other vm arguments
        let mut seq_vm_args = vm_args.to_owned();
        seq_vm_args.push_str(" ");
        seq_vm_args.push_str(&config.jdk.vmargs.sequencer);

        let handle = server::create_server_thread(
            barrier.clone(),
            stop_sign.clone(),
            config.clone(),
            seq_conn.clone(),
            db_name.to_owned(), seq_vm_args,
            true,
            tx.clone(),
            action
        );
        threads.push(handle);
    }

    // Add other vm arguments for clients
    let mut client_vm_args = vm_args.to_owned();
    client_vm_args.push_str(" ");
    client_vm_args.push_str(&config.jdk.vmargs.clients);

    // Create client connections
    for client_conn in &client_list {
        let handle = client::create_client_thread(
            barrier.clone(),
            config.clone(),
            client_conn.clone(),
            client_vm_args.clone(),
            tx.clone(),
            action,
            report_dir.clone()
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
                    info!("All clients finished properly. Stopping server threads...");

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

    info!("All threads exits properly.");

    Ok(client_results)
}