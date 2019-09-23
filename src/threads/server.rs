
use std::sync::{Arc, Barrier, RwLock};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use log::*;

use crate::error::Result;
use crate::config::Config;
use crate::connections::{Server, Action, ConnectionInfo};
use super::{ThreadResult, CHECKING_INTERVAL};

pub fn create_server_thread(barrier: Arc<Barrier>,
        stop_sign: Arc<RwLock<bool>>, config: Config,
        conn_info: ConnectionInfo, db_name: String,
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
        if server.is_sequencer() {
            debug!("The sequencer finished.");
        } else {
            debug!("Server {} finished.", server.id());
        }
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

    if server.is_sequencer() {
        debug!("The sequencer is ready.");
    } else {
        debug!("Server {} is ready.", server.id());
    }

    // Wait for all servers ready
    barrier.wait();

    if server.id() == 0 {
        info!("All servers are ready.");
    }

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