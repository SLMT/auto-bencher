
pub mod init_env;
pub mod load;
pub mod benchmark;
pub mod all_execute;
pub mod pull;

use log::*;

use crate::error::{Result, BenchError};
use crate::parameters::Parameter;
use crate::config::Config;
use crate::command;
use crate::connections::{Action, ConnectionInfo};
use crate::threads;

fn run(config: &Config, parameter: &Parameter,
        db_name: &str, action: Action, report_dir: Option<String>) -> Result<Vec<Option<u32>>> {
    
    // Generate connection information (ip, port)
    let (sequencer, server_list, client_list) =
        generate_connection_list(config, parameter, action)?;
    
    // Prepare the bench dir
    let vm_args = crate::preparation::prepare_bench_dir(
        &config, parameter, &sequencer, &server_list, &client_list)?;

    info!("Connecting to machines...");

    info!("Killing existing benchmarker processes...");
    if let Some(ref seq) = sequencer {
        kill_benchmarker(config, seq)?;
    }
    for server in &server_list {
        kill_benchmarker(config, server)?;
    }
    for client in &client_list {
        kill_benchmarker(config, client)?;
    }

    threads::run_in_threads(config, db_name, action, report_dir,
        &vm_args, sequencer, server_list, client_list)
}

fn generate_connection_list(config: &Config, parameter: &Parameter, action: Action)
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
    let client_list = if let Action::Loading = action {
        ConnectionInfo::generate_connection_list(
            &config.machines.clients,
            1,
            max_client_per_machine
        )?
    } else {
        ConnectionInfo::generate_connection_list(
            &config.machines.clients,
            client_count,
            max_client_per_machine
        )?
    };

    Ok((sequencer, server_list, client_list))
}

fn kill_benchmarker(config: &Config, machine: &ConnectionInfo) -> Result<()> {
    let result = command::ssh(
        &config.system.user_name,
        &machine.ip,
        "pkill -f benchmarker"
    );
    match result {
        Err(BenchError::CommandFailedOnRemote(_, _, 1, _)) =>
                debug!("No existing process is found on '{}'", machine.ip),
        Err(e) => return Err(e),
        _ => {}
    }
    Ok(())
}
