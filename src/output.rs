use crate::input::Person;
use chrono::{NaiveDate, TimeDelta};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

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
}

impl Display for Schedule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for turn in &self.turns {
            let length = turn.end - turn.start;
            writeln!(
                f,
                "{}\t{} - {} ({} days)",
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
