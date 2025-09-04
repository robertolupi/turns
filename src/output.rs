use crate::input::Person;
use chrono::{NaiveDate, TimeDelta};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScheduleError {
    #[error("No one is available on {0}")]
    NoOneAvailable(NaiveDate),
}

#[derive(Debug)]
pub struct Assignment {
    pub(crate) person: usize,
    pub(crate) start: NaiveDate,
    pub(crate) end: NaiveDate,
}

#[derive(Debug)]
pub struct Schedule {
    pub(crate) people: Vec<Person>,
    pub(crate) turns: Vec<Assignment>,
}

#[derive(Debug)]
pub struct Load<'a> {
    pub(crate) days: HashMap<&'a Person, TimeDelta>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct YamlAssignment<'a> {
    #[serde(borrow)]
    pub(crate) person: &'a str,
    pub(crate) start: NaiveDate,
    pub(crate) end: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct YamlSchedule<'a> {
    #[serde(borrow)]
    pub(crate) schedule: Vec<YamlAssignment<'a>>,
}

impl Schedule {
    fn load(&self) -> Load<'_> {
        let mut days: HashMap<&Person, TimeDelta> = HashMap::new();
        for turn in &self.turns {
            let person = &self.people[turn.person];
            let length = turn.end - turn.start;
            *days.entry(person).or_insert(TimeDelta::zero()) += length;
        }
        Load { days }
    }

    pub(crate) fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        let assignments: Vec<YamlAssignment> = self
            .turns
            .iter()
            .map(|turn| {
                let person = &self.people[turn.person];
                YamlAssignment {
                    person: &person.id,
                    start: turn.start,
                    end: turn.end,
                }
            })
            .collect();

        let yaml_schedule = YamlSchedule {
            schedule: assignments,
        };

        serde_yaml::to_string(&yaml_schedule)
    }
}

impl Display for Schedule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for turn in &self.turns {
            let length = turn.end - turn.start;
            writeln!(
                f,
                "{}	{} - {} ({} days)",
                self.people[turn.person].name,
                turn.start,
                turn.end,
                length.num_days()
            )?;
        }
        
        writeln!(f, "\nLoad summary:")?;
        let load = self.load();
        for (person, days) in load.days {
            writeln!(f, "{}: {} days", person.name, days.num_days())?;
        }
        Ok(())
    }
}