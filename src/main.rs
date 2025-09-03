mod algo;
mod config;
mod input;
mod output;

use clap::{Parser, arg};
use std::path::PathBuf;
use crate::input::Person;
use env_logger::Builder;
use log::LevelFilter;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value = "turns.yaml")]
    config: PathBuf,

    #[arg(short, long, default_value = "0")]
    verbose: u8,
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

    let people: Vec<Person> = cfg.people.values().map(|p| p.into()).collect();
    let start = cfg.schedule.from;
    let end = cfg.schedule.to;

    let output = match cfg.schedule.algo {
        config::Algo::RoundRobin { turn_length_days } => {
            algo::roundrobin::schedule(people, start, end, turn_length_days)
        }
        config::Algo::Greedy {
            turn_length_days,
            preference_weight,
        } => algo::greedy::schedule(people, start, end, turn_length_days, preference_weight),
        config::Algo::Balanced {
            min_turn_days,
            max_turn_days,
        } => algo::balanced::schedule(people, start, end, min_turn_days, max_turn_days),
    };

    match output {
        Ok(schedule) => println!("{}", schedule),
        Err(e) => {
            eprintln!("Error generating schedule: {}", e);
            std::process::exit(1);
        }
    }
}
