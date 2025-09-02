use std::collections::{BinaryHeap, HashMap};
use chrono::{Days, NaiveDate, TimeDelta};
use crate::config::Config;
use crate::input::Person;
use crate::output::{Assignment, Schedule};

pub fn schedule(
    people: Vec<Person>,
    start: NaiveDate,
    end: NaiveDate,
    turn_length_days: u8) -> Result<Schedule, String> {
    let mut turns = vec![];

    let mut current_day = start;
    let mut load: Vec<TimeDelta> = people.iter().map(|_| TimeDelta::zero()).collect();
    let mut assignee: Option<usize> = None;
    while current_day < end {
        // build a Binary Heap of people with the lowest load
        let mut candidate: usize = 0;
        let mut min_load = TimeDelta::MAX;
        for (i, person) in people.iter().enumerate() {
            if people[i].ooo.contains(&current_day) {
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
            return Err(format!("No one is available on {}", current_day));
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