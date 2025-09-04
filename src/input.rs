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
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) ooo: HashSet<NaiveDate>,
    pub(crate) preferences: HashMap<NaiveDate, PreferenceType>,
}

impl Hash for Person {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl From<(&String, &config::Person)> for Person {
    fn from(value: (&String, &config::Person)) -> Self {
        let (id, p) = value;
        let mut ooo = HashSet::new();

        if let Some(ooo_vec) = &p.ooo {
            for ooo_entry in ooo_vec {
                match ooo_entry {
                    Ooo::Day(date) => {
                        info!("{} is Ooo on {}", p.name, date);
                        ooo.insert(*date);
                    }
                    Ooo::Period { from, to } => {
                        let mut current = *from;
                        while current <= *to {
                            info!("{} is Ooo on {}", p.name, current);
                            ooo.insert(current);
                            current = current.succ_opt().unwrap();
                        }
                    }
                }
            }
        }

        let mut preferences = HashMap::new();
        if let Some(pref_vec) = &p.preferences {
            for pref_entry in pref_vec {
                match pref_entry {
                    Preference::Want(date) => {
                        info!("{} wants to be on call on {}", p.name, date);
                        preferences.insert(*date, PreferenceType::Want);
                    }
                    Preference::NotWant(date) => {
                        info!("{} does not want to be on call on {}", p.name, date);
                        preferences.insert(*date, PreferenceType::NotWant);
                    }
                }
            }
        }

        Person {
            id: id.clone(),
            name: p.name.clone(),
            ooo,
            preferences,
        }
    }
}
