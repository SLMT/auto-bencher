
use std::path::Path;
use std::fs::File;

use log::*;
use clap::{ArgMatches, Arg, App, SubCommand};
use chrono::prelude::*;

use crate::error::Result;
use crate::config::Config;
use crate::parameters::{Parameter, ParameterList};
use crate::connections::Action;

pub fn get_sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("bench")
                .arg(Arg::with_name("DB NAME")
                    .help("The name of the database that holds the testbed")
                    .required(true)
                    .index(1))
                .arg(Arg::with_name("PARAMETER FILE")
                    .help("The parameters of running the benchmarks")
                    .required(true)
                    .index(2))
                .about("running the benchmarks using the given parameters")
}

pub fn execute(config: &Config, args: &ArgMatches) -> Result<()> {
    let db_name = args.value_of("DB NAME").unwrap();
    let param_file = args.value_of("PARAMETER FILE").unwrap();
    
    info!("Preparing for running benchmarks...");
    info!("Using parameter file '{}'", param_file);

    // TODO: Check if the database exists

    // Read the parameter file
    let param_list = ParameterList::from_file(Path::new(param_file))?;
    let param_list = param_list.to_vec();

    // Prepare for the final report
    std::fs::create_dir_all("reports")?;
    let mut writer = get_report_writer()?;
    write_csv_header(&mut writer, &param_list[0])?;

    // Running jobs
    for job_id in 0 .. param_list.len() {
        info!("Running job {}...", job_id);

        let throughput_str = match super::run_server_and_client(
            config, &param_list[job_id],
            &db_name, Action::Benchmarking
        ) {
            Ok(th) => {
                info!("Job {} finished successfully.", job_id);
                th.unwrap().to_string()
            },
            Err(e) => {
                info!("Job {} finished with an error: {}", job_id, e);
                "error".to_owned()
            }
        };

        info!("Writing the result to the report...");
        write_report(&mut writer, &param_list[job_id], &throughput_str)?;
        info!("Finished writing the result of job {}", job_id);
    }

    // Show the final result (where is the database, the size...)
    info!("Benchmarking finished.");

    Ok(())
}

fn get_report_writer() -> Result<csv::Writer<File>> {
    let dt = Local::now();
    let dt_str = dt.format("%Y_%m_%d_%H_%M_%S").to_string();
    let path = format!("reports/{}.csv", dt_str);
    Ok(csv::Writer::from_path(path)?)
}

fn write_csv_header(writer: &mut csv::Writer<File>,
        parameter: &Parameter) -> Result<()> {
    
    let properties = parameter.get_properties();
    let mut headers: Vec<_> = properties.iter()
        .map(|p| p.split(".").last().unwrap()).collect();
    headers.push("throughput");
    writer.write_record(headers)?;

    Ok(())
}

fn write_report(writer: &mut csv::Writer<File>,
        parameter: &Parameter, throughput_str: &str) -> Result<()> {
    
    let mut values = parameter.get_properties_values();
    values.push(throughput_str);
    writer.write_record(values)?;
    writer.flush()?;

    Ok(())
}