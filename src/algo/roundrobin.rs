use crate::input::Person;
use crate::output::{Assignment, Schedule};
use chrono::{Days, NaiveDate};

use crate::output::ScheduleError;

pub fn schedule( 
                people: Vec<Person>,
                start: NaiveDate,
                end: NaiveDate,
                turn_length_days: u8) -> Result<Schedule, ScheduleError> {
    let mut turns = vec![];

    let mut current_day = start;
    let mut assignee: usize = 0;
    while current_day < end {
        let mut candidate = assignee;
        while people[candidate].ooo.contains(&current_day) {
            candidate = (candidate + 1) % people.len();
            if candidate == assignee {
                return Err(ScheduleError::NoOneAvailable(current_day));
            }
        }
        assignee = candidate;
        let start = current_day.clone();
        let last_day = current_day
            .checked_add_days(Days::new(turn_length_days.into()))
            .unwrap();
        // check if the candidate is available for the whole turn
        while current_day < last_day
            && current_day < end
            && !people[candidate].ooo.contains(&current_day)
        {
            current_day = current_day.succ_opt().unwrap();
        }
        turns.push(Assignment {
            person: candidate,
            start: start,
            end: current_day.clone(),
        });
        assignee = (assignee + 1) % people.len();
    }

    Ok(Schedule {
        people: people,
        turns: turns,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Person;
    use chrono::NaiveDate;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_simple_schedule() {
        let people = vec![
            Person {
                name: "Alice".to_string(),
                ooo: HashSet::new(),
                preferences: HashMap::new(),
            },
            Person {
                name: "Bob".to_string(),
                ooo: HashSet::new(),
                preferences: HashMap::new(),
            },
        ];
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let schedule = schedule(people, start, end, 2).unwrap();
        assert_eq!(schedule.turns.len(), 2);
        assert_eq!(schedule.turns[0].person, 0);
        assert_eq!(schedule.turns[1].person, 1);
    }

    #[test]
    fn test_with_ooo() {
        let mut ooo = HashSet::new();
        ooo.insert(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let people = vec![
            Person {
                name: "Alice".to_string(),
                ooo: ooo,
                preferences: HashMap::new(),
            },
            Person {
                name: "Bob".to_string(),
                ooo: HashSet::new(),
                preferences: HashMap::new(),
            },
        ];
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let schedule = schedule(people, start, end, 2).unwrap();
        assert_eq!(schedule.turns.len(), 2);
        assert_eq!(schedule.turns[0].person, 1); // Bob starts because Alice is OOO
        assert_eq!(schedule.turns[1].person, 0);
    }

    #[test]
    fn test_no_one_available() {
        let mut ooo = HashSet::new();
        ooo.insert(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        let people = vec![
            Person {
                name: "Alice".to_string(),
                ooo: ooo.clone(),
                preferences: HashMap::new(),
            },
            Person {
                name: "Bob".to_string(),
                ooo: ooo.clone(),
                preferences: HashMap::new(),
            },
        ];
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let result = schedule(people, start, end, 2);
        assert!(matches!(result, Err(ScheduleError::NoOneAvailable(_))));
    }
}
