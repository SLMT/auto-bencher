
use std::path::{Path, PathBuf};
use std::fs::File;
use std::collections::BTreeMap;

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
    let main_report_dir = create_report_dir()?;
    let mut writer = get_main_report_writer(&main_report_dir)?;
    write_csv_header(&mut writer, &param_list[0])?;

    // Running jobs
    for job_id in 0 .. param_list.len() {
        info!("Running job {}...", job_id);

        let job_report_dir = create_job_dir(&main_report_dir, job_id)?;

        let throughput_str = match super::run(
            config, &param_list[job_id],
            &db_name, Action::Benchmarking, Some(job_report_dir.display().to_string())
        ) {
            Ok(ths) => {
                let mut total_throughput = 0;
                for th in ths {
                    total_throughput += th.unwrap();
                }
                info!("Job {} finished successfully.", job_id);
                total_throughput.to_string()
            },
            Err(e) => {
                info!("Job {} finished with an error: {}", job_id, e);
                "error".to_owned()
            }
        };

        info!("Writing the result to the report...");
        aggregate_results(&main_report_dir, job_id)?;
        write_report(&mut writer, job_id, &param_list[job_id], &throughput_str)?;
        info!("Finished writing the result of job {}", job_id);
    }

    // Show the final result (where is the database, the size...)
    info!("Benchmarking finished.");

    Ok(())
}

fn create_report_dir() -> Result<PathBuf> {
    let dt = Local::now();
    let date_str = dt.format("%Y-%m-%d").to_string();
    let time_str = dt.format("%H-%M-%S").to_string();
    let mut report_dir_path = PathBuf::new();
    report_dir_path.push("reports");
    report_dir_path.push(date_str);
    report_dir_path.push(time_str);
    std::fs::create_dir_all(&report_dir_path)?;
    Ok(report_dir_path)
}

fn create_job_dir(main_report_dir: &Path, job_id: usize) -> Result<PathBuf> {
    let job_dir = main_report_dir.join(&format!("job-{}", job_id));
    std::fs::create_dir_all(&job_dir)?;
    Ok(job_dir)
}

fn aggregate_results(main_dir: &Path, job_id: usize) -> Result<()> {
    // Prepare variables
    let mut timeline: BTreeMap<usize, usize> = BTreeMap::new();

    // Open each csv files
    let job_dir = main_dir.join(&format!("job-{}", job_id));
    for entry in std::fs::read_dir(job_dir)? {
        let filepath = entry?.path();
        if filepath.is_file() && filepath.extension().unwrap() == "csv" {
            
            debug!("Reading {}...", filepath.to_str().unwrap());

            let mut reader = csv::Reader::from_path(&filepath)?;

            // Read each row
            for result in reader.records() {
                let record = result?;
                let time: usize = record.get(0).unwrap().trim().parse()?;
                let throughput: usize = record.get(1).unwrap().trim().parse()?;

                let total = timeline.entry(time).or_default();
                *total += throughput;
            }

            debug!("Finished parsing {}.", filepath.to_str().unwrap());
        }
    }

    // Write to an output file
    let timeline_filename = main_dir.join(&format!("job-{}-timeline.csv", job_id));
    let mut writer = csv::Writer::from_path(timeline_filename)?;
    writer.write_record(&["time", "throughput"])?;
    for (time, throughput) in timeline {
        writer.write_record(&[time.to_string(), throughput.to_string()])?;
    }
    writer.flush()?;

    Ok(())
}

fn get_main_report_writer(report_dir: &Path) -> Result<csv::Writer<File>> {
    let file_path = report_dir.join("throughput.csv");
    Ok(csv::Writer::from_path(file_path)?)
}

fn write_csv_header(writer: &mut csv::Writer<File>,
        parameter: &Parameter) -> Result<()> {
    
    let properties = parameter.get_properties();
    let mut headers = vec!["job_id"];
    let mut params: Vec<_> = properties.iter()
        .map(|p| p.split(".").last().unwrap()).collect();
    headers.append(&mut params);
    headers.push("throughput");
    writer.write_record(headers)?;

    Ok(())
}

fn write_report(writer: &mut csv::Writer<File>, job_id: usize,
        parameter: &Parameter, throughput_str: &str) -> Result<()> {
    let job_id = job_id.to_string();
    let mut values: Vec<&str> = vec![];
    let mut params = parameter.get_properties_values();

    values.push(&job_id);
    values.append(&mut params);
    values.push(throughput_str);
    writer.write_record(values)?;
    writer.flush()?;

    Ok(())
}