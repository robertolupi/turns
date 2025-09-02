use crate::config::Config;
use crate::input::Person;
use crate::output::{Assignment, Schedule};
use chrono::{Days, NaiveDate};
use std::fs::metadata;

pub fn schedule( 
                people: Vec<Person>,
                start: NaiveDate,
                end: NaiveDate,
                turn_length_days: u8) -> Result<Schedule, String> {
    let mut turns = vec![];

    let mut current_day = start;
    let mut assignee: usize = 0;
    while current_day < end {
        let mut candidate = assignee;
        while people[candidate].ooo.contains(&current_day) {
            candidate = (candidate + 1) % people.len();
            if candidate == assignee {
                return Err(format!("No one is available on {}", current_day));
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
