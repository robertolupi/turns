use crate::input::{Person, PreferenceType};
use crate::output::{Assignment, Schedule, ScheduleError};
use chrono::{Days, NaiveDate, TimeDelta};

fn is_ooo_for_turn(person: &Person, start_date: NaiveDate, end_date: NaiveDate) -> bool {
    let mut current_date = start_date;
    while current_date < end_date {
        if person.ooo.contains(&current_date) {
            return true;
        }
        current_date = current_date.succ_opt().unwrap();
    }
    false
}

fn calculate_load_variance(load: &[TimeDelta]) -> f64 {
    let n = load.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let mean = load.iter().map(|d| d.num_seconds() as f64).sum::<f64>() / n;
    load.iter()
        .map(|d| {
            let diff = d.num_seconds() as f64 - mean;
            diff * diff
        })
        .sum::<f64>()
        / n
}

pub fn schedule(
    people: Vec<Person>,
    start: NaiveDate,
    end: NaiveDate,
    min_turn_days: u8,
    max_turn_days: u8,
) -> Result<Schedule, ScheduleError> {
    let mut turns = vec![];
    let mut current_day = start;
    let mut load: Vec<TimeDelta> = people.iter().map(|_| TimeDelta::zero()).collect();
    let mut last_assignee: Option<usize> = None;

    while current_day < end {
        let mut best_choice: Option<(usize, NaiveDate, f64)> = None;

        for (i, person) in people.iter().enumerate() {
            if Some(i) == last_assignee {
                continue;
            }

            for turn_len in min_turn_days..=max_turn_days {
                let turn_end = std::cmp::min(
                    end,
                    current_day
                        .checked_add_days(Days::new(turn_len as u64))
                        .unwrap(),
                );

                if is_ooo_for_turn(person, current_day, turn_end) {
                    continue;
                }

                let mut has_want = false;
                let mut has_not_want = false;
                let mut d = current_day;
                while d < turn_end {
                    if let Some(pref) = person.preferences.get(&d) {
                        match pref {
                            PreferenceType::Want => has_want = true,
                            PreferenceType::NotWant => has_not_want = true,
                        }
                    }
                    d = d.succ_opt().unwrap();
                }

                let preference_group = if has_want {
                    0
                } else if has_not_want {
                    2
                } else {
                    1
                };

                let mut next_load = load.clone();
                next_load[i] += turn_end - current_day;
                let variance = calculate_load_variance(&next_load);

                let current_best_group = if let Some((_, _, group)) = best_choice {
                    group as i32
                } else {
                    3
                };

                if preference_group < current_best_group {
                    best_choice = Some((i, turn_end, preference_group as f64));
                } else if preference_group == current_best_group {
                    let current_best_variance = if let Some((_, _, _)) = best_choice {
                        calculate_load_variance(&load)
                    } else {
                        f64::MAX
                    };
                    if variance < current_best_variance {
                        best_choice = Some((i, turn_end, preference_group as f64));
                    }
                }
            }
        }

        if let Some((assignee, turn_end, _)) = best_choice {
            turns.push(Assignment {
                person: assignee,
                start: current_day,
                end: turn_end,
            });
            load[assignee] += turn_end - current_day;
            current_day = turn_end;
            last_assignee = Some(assignee);
        } else {
            return Err(ScheduleError::NoOneAvailable(current_day));
        }
    }

    Ok(Schedule { people, turns })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Person, PreferenceType};
    use chrono::NaiveDate;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_simple_balanced_schedule() {
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
        let end = NaiveDate::from_ymd_opt(2025, 1, 11).unwrap(); // 10 days
        let schedule = schedule(people, start, end, 3, 7).unwrap();

        // Expect Alice: 5 days, Bob: 5 days
        let alice_load = schedule.turns.iter().filter(|t| t.person == 0).map(|t| (t.end - t.start).num_days()).sum::<i64>();
        let bob_load = schedule.turns.iter().filter(|t| t.person == 1).map(|t| (t.end - t.start).num_days()).sum::<i64>();

        assert_eq!(alice_load, 5);
        assert_eq!(bob_load, 5);
    }

    #[test]
    fn test_balanced_with_preferences() {
        let mut alice_prefs = HashMap::new();
        alice_prefs.insert(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            PreferenceType::Want,
        );

        let people = vec![
            Person {
                name: "Alice".to_string(),
                ooo: HashSet::new(),
                preferences: alice_prefs,
            },
            Person {
                name: "Bob".to_string(),
                ooo: HashSet::new(),
                preferences: HashMap::new(),
            },
        ];
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let schedule = schedule(people, start, end, 1, 3).unwrap();
        assert_eq!(schedule.turns[0].person, 0); // Alice gets the first turn
    }
}