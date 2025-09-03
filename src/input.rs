use crate::config;
use crate::config::{Ooo, Preference};
use chrono::NaiveDate;
use log::info;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum PreferenceType {
    Want,
    NotWant,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Person {
    pub(crate) name: String,
    pub(crate) ooo: HashSet<NaiveDate>,
    pub(crate) preferences: HashMap<NaiveDate, PreferenceType>,
}

impl Hash for Person {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl From<&config::Person> for Person {
    fn from(value: &config::Person) -> Self {
        let mut ooo = HashSet::new();

        if let Some(ooo_vec) = &value.ooo {
            for ooo_entry in ooo_vec {
                match ooo_entry {
                    Ooo::Day(date) => {
                        info!("{} is Ooo on {}", value.name, date);
                        ooo.insert(*date);
                    }
                    Ooo::Period { from, to } => {
                        let mut current = *from;
                        while current <= *to {
                            info!("{} is Ooo on {}", value.name, current);
                            ooo.insert(current);
                            current = current.succ_opt().unwrap();
                        }
                    }
                }
            }
        }

        let mut preferences = HashMap::new();
        if let Some(pref_vec) = &value.preferences {
            for pref_entry in pref_vec {
                match pref_entry {
                    Preference::Want(date) => {
                        info!("{} wants to be on call on {}", value.name, date);
                        preferences.insert(*date, PreferenceType::Want);
                    }
                    Preference::NotWant(date) => {
                        info!("{} does not want to be on call on {}", value.name, date);
                        preferences.insert(*date, PreferenceType::NotWant);
                    }
                }
            }
        }

        Person {
            name: value.name.clone(),
            ooo,
            preferences,
        }
    }
}
