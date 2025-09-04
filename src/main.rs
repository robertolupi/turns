mod algo;
mod config;
mod input;
mod output;

use clap::{Parser, arg};
use std::path::PathBuf;
use crate::input::Person;
use env_logger::Builder;
use log::LevelFilter;
use std::collections::HashMap;
use chrono::TimeDelta;
use crate::output::YamlSchedule;
use std::fs;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value = "turns.yaml")]
    config: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long)]
    previous: Option<PathBuf>,

    #[arg(short, long, default_value = "0")]
    verbose: u8,
}

fn calculate_initial_load(previous_schedule_path: &PathBuf) -> Result<HashMap<String, TimeDelta>, String> {
    let content = fs::read_to_string(previous_schedule_path)
        .map_err(|e| format!("Failed to read previous schedule file: {}", e))?;
    let previous_schedule: YamlSchedule = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse previous schedule file: {}", e))?;

    let mut initial_load = HashMap::new();
    for assignment in previous_schedule.schedule {
        let duration = assignment.end - assignment.start;
        *initial_load.entry(assignment.person.to_string()).or_insert(TimeDelta::zero()) += duration;
    }
    Ok(initial_load)
}

fn main() {
    let args = Cli::parse();

    let log_level = match args.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Builder::new()
        .filter(None, log_level)
        .init();

    let cfg = match config::parse(&args.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error parsing config: {}", e);
            std::process::exit(1);
        }
    };

    let initial_load = if let Some(previous_path) = &args.previous {
        match calculate_initial_load(previous_path) {
            Ok(load) => Some(load),
            Err(e) => {
                eprintln!("Error processing previous schedule: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let people: Vec<Person> = cfg.people.iter().map(|p| p.into()).collect();
    let start = cfg.schedule.from;
    let end = cfg.schedule.to;

    let output = match cfg.schedule.algo {
        config::Algo::RoundRobin { turn_length_days } => {
            algo::roundrobin::schedule(people, start, end, turn_length_days, initial_load)
        }
        config::Algo::Greedy {
            turn_length_days,
            preference_weight,
        } => algo::greedy::schedule(people, start, end, turn_length_days, preference_weight, initial_load),
        config::Algo::Balanced {
            min_turn_days,
            max_turn_days,
        } => algo::balanced::schedule(people, start, end, min_turn_days, max_turn_days, initial_load),
    };

    match output {
        Ok(schedule) => match schedule.to_yaml() {
            Ok(yaml) => {
                if let Some(output_path) = args.output {
                    if let Err(e) = std::fs::write(output_path, yaml) {
                        eprintln!("Error writing to output file: {}", e);
                        std::process::exit(1);
                    }
                } else {
                    println!("{}", yaml);
                }
            }
            Err(e) => {
                eprintln!("Error serializing to YAML: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error generating schedule: {}", e);
            std::process::exit(1);
        }
    }
}
