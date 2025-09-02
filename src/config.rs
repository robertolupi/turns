use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OOO {
    Day(NaiveDate),
    Period { from: NaiveDate, to: NaiveDate },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub(crate) name: String,
    pub(crate) ooo: Option<Vec<OOO>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Algo {
    RoundRobin { turn_length_days: u8 },
    Greedy { turn_length_days: u8 },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub(crate) from: NaiveDate,
    pub(crate) to: NaiveDate,
    pub(crate) algo: Algo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub(crate) people: HashMap<String, Person>,
    pub(crate) schedule: Schedule,
}

pub fn parse(config_file: PathBuf) -> Result<Config, String> {
    config_file
        .as_path()
        .to_str()
        .ok_or("Invalid config file path".to_string())
        .and_then(|path| std::fs::read_to_string(path).map_err(|e| e.to_string()))
        .and_then(|content| serde_yaml::from_str(&content).map_err(|e| e.to_string()))
}
