
use std::fs;

use log::*;
use clap::{ArgMatches, Arg, App, SubCommand};

use crate::error::Result;
use crate::config::Config;
use crate::command;

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("pull")
                .arg(Arg::with_name("PATTERN")
                    .help("The pattern of file names of files that you wish to pull")
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("SEPARATE")
                    .long("separate")
                    .short("s")
                    .help("Separates the pulled files according to their source ip"))
                .arg(Arg::with_name("IGNORE ERROR")
                    .long("ignore-error")
                    .short("i")
                    .help("If there is an error happens in a job, do not stop and proceed to the next job."))
                .about("pulls the files whose file name matching the given pattern")
}

pub fn execute(config: &Config, args: &ArgMatches) -> Result<()> {
    let pattern = args.value_of("PATTERN").unwrap();
    let is_separated = args.is_present("SEPARATE");
    let ignore_error = args.is_present("IGNORE ERROR");
    let remote_path = format!("{}/{}", &config.system.remote_work_dir, pattern);

    let local_dir = "pulls";
    fs::create_dir_all(&local_dir)?;

    for ip in &config.machines.all {
        info!("Pulling files from {}...", &ip);
        let result = if is_separated {
            let local_path = format!("{}/{}", local_dir, ip);
            fs::create_dir_all(&local_path)?;
            command::scp_from(false, &config.system.user_name, &ip, &remote_path, &local_path)
        } else {
            command::scp_from(false, &config.system.user_name, &ip, &remote_path, &local_dir)
        };

        if let Err(e) = result {
            if ignore_error {
                warn!("{}", e);
            } else {
                return Err(e);
            }
        }
    }

    Ok(())
}
