
mod config;
mod init_env;

use clap::{Arg, ArgMatches, App};

use config::Config;

fn main() {
    let matches = App::new("Auto Bencher")
                       .version("1.0")
                       .author("Yu-Shan Lin <yslin@datalab.cs.nthu.edu.tw>")
                       .about("Automatically run benchmarking using VanillaBench or ElasqlBench")
                       .arg(Arg::with_name("config")
                            .short("c")
                            .long("config")
                            .value_name("FILE")
                            .help("Sets the path to a config file")
                            .takes_value(true))
                       .subcommand(init_env::get_sub_command())
                       .get_matches();
    
    match execute(matches) {
        Ok(_) => println!("Auto Bencher finishes."),
        Err(s) => eprintln!("Auto Bencher exits with an error:\n{}", s)
    }
}

fn execute(matches: ArgMatches) -> Result<(), String> {
     // Read the config
    let config_file_path = matches.value_of("config").unwrap_or("config.toml");
    let config = Config::from_file(&config_file_path)?;

    // Choose action according to the sub command
    if let Some(matches) = matches.subcommand_matches("init-env") {
        init_env::execute(&config, matches);
    }

    Ok(())
}