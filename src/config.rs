use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid config file path: {0}")]
    InvalidPath(PathBuf),
    #[error("Failed to read config file: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    Parse(#[from] serde_yaml::Error),
    #[error("Person name cannot be empty")]
    EmptyPersonName,
    #[error("Invalid date range: `from` date must be before `to` date")]
    InvalidDateRange,
    #[error("Turn length in days must be positive")]
    InvalidTurnLength,
    #[error("OOO period is invalid for person {person_name}: `from` date must be before `to` date")]
    InvalidOOOPeriod { person_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OOO {
    Day(NaiveDate),
    Period { from: NaiveDate, to: NaiveDate },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Preference {
    Want(NaiveDate),
    NotWant(NaiveDate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub(crate) name: String,
    pub(crate) ooo: Option<Vec<OOO>>,
    pub(crate) preferences: Option<Vec<Preference>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Algo {
    RoundRobin { turn_length_days: u8 },
    Greedy {
        turn_length_days: u8,
        #[serde(default)]
        preference_weight: Option<u8>,
    },
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

impl Config {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.schedule.from >= self.schedule.to {
            return Err(ConfigError::InvalidDateRange);
        }

        match self.schedule.algo {
            Algo::RoundRobin { turn_length_days } | Algo::Greedy { turn_length_days, .. } => {
                if turn_length_days == 0 {
                    return Err(ConfigError::InvalidTurnLength);
                }
            }
        }

        for person in self.people.values() {
            if person.name.is_empty() {
                return Err(ConfigError::EmptyPersonName);
            }
            if let Some(ooos) = &person.ooo {
                for ooo in ooos {
                    if let OOO::Period { from, to } = ooo {
                        if from >= to {
                            return Err(ConfigError::InvalidOOOPeriod {
                                person_name: person.name.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn parse(config_file: &Path) -> Result<Config, ConfigError> {
    if !config_file.exists() || !config_file.is_file() {
        return Err(ConfigError::InvalidPath(config_file.to_path_buf()));
    }
    let content = std::fs::read_to_string(config_file)?;
    let config: Config = serde_yaml::from_str(&content)?;
    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_config_to_tempfile(config: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", config).unwrap();
        file
    }

    #[test]
    fn test_parse_valid_config() {
        let config = r#"
people:
  alice:
    name: Alice
    preferences:
      - !Want 2025-01-10
  bob:
    name: Bob
schedule:
  from: 2025-01-01
  to: 2025-01-31
  algo: !RoundRobin
    turn_length_days: 7
"#;
        let file = write_config_to_tempfile(config);
        let result = parse(file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_person_name() {
        let config = r#"
people:
  alice:
    name: ""
    preferences: []
schedule:
  from: 2025-01-01
  to: 2025-01-31
  algo: !RoundRobin
    turn_length_days: 7
"#;
        let file = write_config_to_tempfile(config);
        let result = parse(file.path());
        assert!(matches!(result, Err(ConfigError::EmptyPersonName)));
    }

    #[test]
    fn test_parse_invalid_date_range() {
        let config = r#"
people:
  alice:
    name: Alice
    preferences: []
schedule:
  from: 2025-01-31
  to: 2025-01-01
  algo: !RoundRobin
    turn_length_days: 7
"#;
        let file = write_config_to_tempfile(config);
        let result = parse(file.path());
        assert!(matches!(result, Err(ConfigError::InvalidDateRange)));
    }

    #[test]
    fn test_parse_invalid_turn_length() {
        let config = r#"
people:
  alice:
    name: Alice
    preferences: []
schedule:
  from: 2025-01-01
  to: 2025-01-31
  algo: !RoundRobin
    turn_length_days: 0
"#;
        let file = write_config_to_tempfile(config);
        let result = parse(file.path());
        assert!(matches!(result, Err(ConfigError::InvalidTurnLength)));
    }

    #[test]
    fn test_parse_invalid_ooo_period() {
        let config = r#"
people:
  alice:
    name: Alice
    ooo:
      - !Period { from: 2025-01-10, to: 2025-01-05 }
    preferences: []
schedule:
  from: 2025-01-01
  to: 2025-01-31
  algo: !RoundRobin
    turn_length_days: 7
"#;
        let file = write_config_to_tempfile(config);
        let result = parse(file.path());
        assert!(matches!(result, Err(ConfigError::InvalidOOOPeriod { .. })));
    }

    #[test]
    fn test_parse_non_existent_file() {
        let path = PathBuf::from("non_existent_file.yaml");
        let result = parse(&path);
        assert!(matches!(result, Err(ConfigError::InvalidPath(_))));
    }

    #[test]
    fn test_parse_directory_path() {
        let dir = tempfile::tempdir().unwrap();
        let result = parse(dir.path());
        assert!(matches!(result, Err(ConfigError::InvalidPath(_))));
    }
}
