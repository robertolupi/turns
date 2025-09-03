use crate::input::Person;
use crate::output::{Assignment, Schedule};
use chrono::{Days, NaiveDate, TimeDelta};

use crate::output::ScheduleError;

pub fn schedule(
    people: Vec<Person>,
    start: NaiveDate,
    end: NaiveDate,
    turn_length_days: u8) -> Result<Schedule, ScheduleError> {
    let mut turns = vec![];

    let mut current_day = start;
    let mut load: Vec<TimeDelta> = people.iter().map(|_| TimeDelta::zero()).collect();
    let mut assignee: Option<usize> = None;
    while current_day < end {
        // build a Binary Heap of people with the lowest load
        let mut candidate: usize = 0;
        let mut min_load = TimeDelta::MAX;
        for (i, person) in people.iter().enumerate() {
            if person.ooo.contains(&current_day) {
                continue;
            }
            if Some(i) == assignee {
                continue;
            }
            if load[i] < min_load {
                min_load = load[i];
                candidate = i;
            }
        }
        if min_load == TimeDelta::MAX {
            return Err(ScheduleError::NoOneAvailable(current_day));
        }
        assignee = Some(candidate);
        let start = current_day.clone();
        let last_day = current_day
            .checked_add_days(Days::new(turn_length_days.into()))
            .unwrap();
        while current_day < last_day
            && current_day < end
            && !people[assignee.unwrap()].ooo.contains(&current_day)
        {
            current_day = current_day.succ_opt().unwrap();
        }
        turns.push(Assignment {
            person: assignee.unwrap(),
            start: start,
            end: current_day.clone(),
        });
        load[assignee.unwrap()] = load[assignee.unwrap()] + (current_day - start);
    }
    
    Ok(Schedule{
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
    fn test_load_balancing() {
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
        let end = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();
        let schedule = schedule(people, start, end, 3).unwrap();
        // Expected schedule:
        // Alice: 1/1 - 1/4 (3 days)
        // Bob: 1/4 - 1/7 (3 days)
        // Alice: 1/7 - 1/10 (3 days)
        assert_eq!(schedule.turns.len(), 3);
        assert_eq!(schedule.turns[0].person, 0);
        assert_eq!(schedule.turns[1].person, 1);
        assert_eq!(schedule.turns[2].person, 0);
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
