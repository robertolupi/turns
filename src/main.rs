mod algo;
mod config;
mod input;
mod output;

use clap::{Parser, arg};
use std::path::PathBuf;
use crate::input::Person;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value = "turns.yaml")]
    config: PathBuf,

    #[arg(short, long, default_value = "0")]
    verbose: u8,
}

fn main() {
    let args = Cli::parse();

    let cfg = config::parse(args.config);
    println!("{:?}", cfg);

    if cfg.is_err() {
        return;
    }

    let cfg = cfg.unwrap();

    let people: Vec<Person> = cfg.people.iter().map(|(_, p)| p.into()).collect();
    let start = cfg.schedule.from;
    let end = cfg.schedule.to;

    let output = match cfg.schedule.algo {
        config::Algo::RoundRobin { turn_length_days } => {
            algo::roundrobin::schedule(people, start, end, turn_length_days)
        },
        config::Algo::Greedy { turn_length_days } => {
            algo::greedy::schedule(people, start, end, turn_length_days)
        }
    };

    println!("{}", output.map_err(|e| e.to_string()).unwrap());
}
