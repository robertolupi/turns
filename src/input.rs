use crate::config;
use crate::config::OOO;
use chrono::NaiveDate;
use log::info;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Person {
    pub(crate) name: String,
    pub(crate) ooo: HashSet<NaiveDate>,
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
                    OOO::Day(date) => {
                        info!("{} is OOO on {}", value.name, date);
                        ooo.insert(date.clone());
                    }
                    OOO::Period { from, to } => {
                        let mut current = from.clone();
                        while current <= *to {
                            info!("{} is OOO on {}", value.name, current);
                            ooo.insert(current.clone());
                            current = current.succ_opt().unwrap();
                        }
                    }
                }
            }
        }
        Person {
            name: value.name.clone(),
            ooo: ooo,
        }
    }
}
