use crate::input::{Person, PreferenceType};
use crate::output::{Assignment, Schedule};
use chrono::{Days, NaiveDate, TimeDelta};
use log::{debug, info, trace};
use std::collections::HashMap;

use crate::output::ScheduleError;

fn is_ooo_for_turn(person: &Person, start_date: NaiveDate, end_date: NaiveDate) -> bool {
    let mut current_date = start_date;
    while current_date < end_date {
        if person.ooo.contains(&current_date) {
            trace!("{} is OOO on {}", person.name, current_date);
            return true;
        }
        current_date = current_date.succ_opt().unwrap();
    }
    false
}

pub fn schedule(
    people: Vec<Person>,
    start: NaiveDate,
    end: NaiveDate,
    turn_length_days: u8,
    _preference_weight: Option<u8>,
    initial_load: Option<HashMap<String, TimeDelta>>,
) -> Result<Schedule, ScheduleError> {
    let mut turns = vec![];
    let mut current_day = start;
    let mut load: Vec<TimeDelta> = people
        .iter()
        .map(|p| {
            if let Some(ref il) = initial_load {
                il.get(&p.id).cloned().unwrap_or(TimeDelta::zero())
            } else {
                TimeDelta::zero()
            }
        })
        .collect();
    let mut last_assignee: Option<usize> = None;

    info!("Starting greedy schedule generation");
    trace!("Initial load: {:?}", load);

    while current_day < end {
        let turn_end_date = std::cmp::min(
            end,
            current_day
                .checked_add_days(Days::new(turn_length_days.into()))
                .unwrap(),
        );
        debug!("Planning turn from {} to {}", current_day, turn_end_date);

        let mut want_candidates = vec![];
        let mut neutral_candidates = vec![];
        let mut not_want_candidates = vec![];

        for (i, person) in people.iter().enumerate() {
            if Some(i) == last_assignee {
                trace!("Skipping {} (last assignee)", person.name);
                continue;
            }

            if is_ooo_for_turn(person, current_day, turn_end_date) {
                debug!("Skipping {} (OOO)", person.name);
                continue;
            }

            let mut has_want = false;
            let mut has_not_want = false;
            let mut d = current_day;
            while d < turn_end_date {
                if let Some(pref) = person.preferences.get(&d) {
                    match pref {
                        PreferenceType::Want => has_want = true,
                        PreferenceType::NotWant => has_not_want = true,
                    }
                }
                d = d.succ_opt().unwrap();
            }

            if has_want {
                trace!("{} has Want preference", person.name);
                want_candidates.push(i);
            } else if has_not_want {
                trace!("{} has NotWant preference", person.name);
                not_want_candidates.push(i);
            } else {
                trace!("{} has neutral preference", person.name);
                neutral_candidates.push(i);
            }
        }
        debug!("Want candidates: {:?}", want_candidates);
        debug!("Neutral candidates: {:?}", neutral_candidates);
        debug!("NotWant candidates: {:?}", not_want_candidates);

        let candidate = if !want_candidates.is_empty() {
            debug!("Choosing from Want candidates");
            want_candidates
                .iter()
                .min_by_key(|&&p| load[p]).copied()
        } else if !neutral_candidates.is_empty() {
            debug!("Choosing from Neutral candidates");
            neutral_candidates
                .iter()
                .min_by_key(|&&p| load[p]).copied()
        } else if !not_want_candidates.is_empty() {
            debug!("Choosing from NotWant candidates");
            not_want_candidates
                .iter()
                .min_by_key(|&&p| load[p]).copied()
        } else {
            None
        };

        if candidate.is_none() {
            return Err(ScheduleError::NoOneAvailable(current_day));
        }

        let assignee = candidate.unwrap();
        last_assignee = Some(assignee);
        info!(
            "Assigning {} to turn {} -> {}",
            people[assignee].name, current_day, turn_end_date
        );

        let actual_turn_end = turn_end_date;

        turns.push(Assignment {
            person: assignee,
            start: current_day,
            end: actual_turn_end,
        });
        load[assignee] += actual_turn_end - current_day;
        trace!("Updated load: {:?}", load);
        current_day = actual_turn_end;
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
        let schedule = schedule(people, start, end, 2, None, None).unwrap();
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
                ooo,
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
        let schedule = schedule(people, start, end, 2, None, None).unwrap();
        assert_eq!(schedule.turns.len(), 2);
        assert_eq!(schedule.turns[0].person, 1); // Bob starts because Alice is OOO
        assert_eq!(schedule.turns[1].person, 0);
    }

    #[test]
    fn test_load_balancing() {
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
        let end = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();
        let schedule = schedule(people, start, end, 3, None, None).unwrap();
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
        let result = schedule(people, start, end, 2, None, None);
        assert!(matches!(result, Err(ScheduleError::NoOneAvailable(_))));
    }

    #[test]
    fn test_with_preferences() {
        let mut alice_prefs = HashMap::new();
        alice_prefs.insert(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            PreferenceType::Want,
        );

        let people = vec![
            Person {
                id: "alice".to_string(),
                name: "Alice".to_string(),
                ooo: HashSet::new(),
                preferences: alice_prefs,
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
        let schedule = schedule(people, start, end, 2, None, None).unwrap();
        assert_eq!(schedule.turns.len(), 2);
        assert_eq!(schedule.turns[0].person, 0); // Alice is chosen because she wants to be on call
        assert_eq!(schedule.turns[1].person, 1);
    }

    #[test]
    fn test_not_want_is_respected() {
        let mut bob_prefs = HashMap::new();
        bob_prefs.insert(
            NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(),
            PreferenceType::NotWant,
        );

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
                preferences: bob_prefs,
            },
             Person {
                id: "charlie".to_string(),
                name: "Charlie".to_string(),
                ooo: HashSet::new(),
                preferences: HashMap::new(),
            },
        ];
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let schedule = schedule(people, start, end, 2, None, None).unwrap();
        assert_eq!(schedule.turns.len(), 2);
        // Alice: 1/1 -> 1/3
        // Charlie: 1/3 -> 1/5
        // Bob is skipped for the second turn because of his NotWant preference.
        assert_eq!(schedule.turns[0].person, 0);
        assert_eq!(schedule.turns[1].person, 2);
    }
}
