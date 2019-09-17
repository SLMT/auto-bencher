
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
    let client_count = if let Action::Loading = action {
        1
    } else {
        client_list.len()
    };
    let thread_count = match sequencer {
        Some(_) => server_list.len() + client_count + 1,
        None => server_list.len() + client_count
    };
    let barrier = Arc::new(Barrier::new(thread_count));

    // Create server connections
    let stop_sign = Arc::new(RwLock::new(false));
    for server_conn in &server_list {
        let handle = server::create_server_thread(
            barrier.clone(),
            stop_sign.clone(),
            config.clone(),
            server_conn.clone(),
            db_name.to_owned(), vm_args.to_owned(),
            false,
            tx.clone(),
            action
        );
        threads.push(handle);
    }

    // Create sequencer connection
    if let Some(seq_conn) = sequencer {
        let handle = server::create_server_thread(
            barrier.clone(),
            stop_sign.clone(),
            config.clone(),
            seq_conn.clone(),
            db_name.to_owned(), vm_args.to_owned(),
            true,
            tx.clone(),
            action
        );
        threads.push(handle);
    }

    // Create client connections
    for client_id in 0 .. client_count {
        let handle = client::create_client_thread(
            barrier.clone(),
            config.clone(),
            client_list[client_id].clone(),
            vm_args.to_owned(),
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
                if client_results.len() >= client_count {
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

    Ok(client_results)
}