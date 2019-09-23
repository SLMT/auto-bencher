
use std::sync::{Arc, Barrier};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use log::*;

use crate::error::{Result};
use crate::config::Config;
use crate::connections::{Client, Action, ConnectionInfo};
use super::{ThreadResult, CHECKING_INTERVAL};

pub fn create_client_thread(barrier: Arc<Barrier>,
        config: Config, conn_info: ConnectionInfo,
        vm_args: String, result_ch: Sender<ThreadResult>,
        action: Action, report_dir: Option<String>)
        -> JoinHandle<()> {
    thread::spawn(move || {
        let client = Client::new(config, conn_info, vm_args);
        let result = match execute_client_thread(&client, barrier, action, report_dir) {
            Err(e) => {
                error!("Client {} (on {}) occurs an error: {}",
                    client.id(), client.ip(), e);
                ThreadResult::Failed
            },
            Ok(th) => ThreadResult::ClientSucceed(th)
        };
        debug!("Client {} finished.", client.id());
        result_ch.send(result).unwrap();
    })
}

fn execute_client_thread(client: &Client, barrier: Arc<Barrier>,
        action: Action, report_dir: Option<String>) -> Result<Option<u32>> {
    client.clean_previous_results()?;
    client.send_bench_dir()?;

    // Wait for the server ready
    barrier.wait(); // prepared
    barrier.wait(); // ready

    if client.id() == 0 {
        info!("Starting clients...");
    }

    client.start(action)?;
    while !client.check_for_finished(action)? {
        thread::sleep(Duration::from_secs(CHECKING_INTERVAL));
    }

    if let Action::Benchmarking = action {
        client.pull_csv(&report_dir.unwrap())?;
        let throughput = client.get_total_throughput()?;
        info!("The total throughput of client {} is {}",
            client.id(), throughput);
        Ok(Some(throughput))
    } else {
        Ok(None)
    }
}