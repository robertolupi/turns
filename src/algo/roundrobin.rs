use crate::input::Person;
use crate::output::{Assignment, Schedule};
use chrono::{Days, NaiveDate, TimeDelta};
use std::collections::HashMap;

use crate::output::ScheduleError;

pub fn schedule(
    people: Vec<Person>,
    start: NaiveDate,
    end: NaiveDate,
    turn_length_days: u8,
    initial_load: Option<HashMap<String, TimeDelta>>,
) -> Result<Schedule, ScheduleError> {
    let mut turns = vec![];

    let mut current_day = start;
    let mut assignee: usize = 0;

    if let Some(il) = initial_load {
        if !il.is_empty() {
            // Find the person who worked the most in the previous schedule
            let last_on_call = il.iter().max_by_key(|(_, v)| *v).map(|(k, _)| k);
            if let Some(last_person_id) = last_on_call {
                if let Some(pos) = people.iter().position(|p| &p.id == last_person_id) {
                    assignee = (pos + 1) % people.len();
                }
            }
        }
    }

    while current_day < end {
        let mut candidate = assignee;
        while people[candidate].ooo.contains(&current_day) {
            candidate = (candidate + 1) % people.len();
            if candidate == assignee {
                return Err(ScheduleError::NoOneAvailable(current_day));
            }
        }
        assignee = candidate;
        let start = current_day;
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
            start,
            end: current_day,
        });
        assignee = (assignee + 1) % people.len();
    }

    Ok(Schedule { people, turns })
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
                id: "alice".to_string(),
                name: "Alice".to_string(),
                ooo: HashSet::new(),
                preferences: HashMap::new(),
            },
            Person {
                id: "bob".to_string(),
                name: "Bob".to_string(),
                ooo: HashSet::new(),
                preferences: HashMap::new(),
            },
        ];
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let schedule = schedule(people, start, end, 2, None).unwrap();
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
                id: "alice".to_string(),
                name: "Alice".to_string(),
                ooo: ooo,
                preferences: HashMap::new(),
            },
            Person {
                id: "bob".to_string(),
                name: "Bob".to_string(),
                ooo: HashSet::new(),
                preferences: HashMap::new(),
            },
        ];
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let schedule = schedule(people, start, end, 2, None).unwrap();
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
                id: "alice".to_string(),
                name: "Alice".to_string(),
                ooo: ooo.clone(),
                preferences: HashMap::new(),
            },
            Person {
                id: "bob".to_string(),
                name: "Bob".to_string(),
                ooo: ooo.clone(),
                preferences: HashMap::new(),
            },
        ];
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let result = schedule(people, start, end, 2, None);
        assert!(matches!(result, Err(ScheduleError::NoOneAvailable(_))));
    }
}
