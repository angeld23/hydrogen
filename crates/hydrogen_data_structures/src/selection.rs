use derive_more::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, IsVariant, Unwrap, TryUnwrap)]
pub enum Selection<T> {
    Whitelist(Vec<T>),
    Blacklist(Vec<T>),
}

impl<T> Selection<T> {
    pub fn none() -> Self {
        Self::Whitelist(vec![])
    }

    pub fn all() -> Self {
        Self::Blacklist(vec![])
    }

    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        match self {
            Selection::Whitelist(values) => values.contains(value),
            Selection::Blacklist(values) => !values.contains(value),
        }
    }

    pub fn get_values(&self) -> &Vec<T> {
        match self {
            Selection::Whitelist(values) => values,
            Selection::Blacklist(values) => values,
        }
    }

    pub fn get_values_mut(&mut self) -> &mut Vec<T> {
        match self {
            Selection::Whitelist(values) => values,
            Selection::Blacklist(values) => values,
        }
    }

    pub fn insert(&mut self, value: T) -> bool
    where
        T: PartialEq,
    {
        let values = self.get_values_mut();

        if !values.contains(&value) {
            values.push(value);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, value: &T) -> Option<T>
    where
        T: PartialEq,
    {
        let values = self.get_values_mut();

        let mut found_index: Option<usize> = None;
        for (i, other_value) in values.iter().enumerate() {
            if other_value == value {
                found_index = Some(i);
                break;
            }
        }

        found_index.map(|i| values.swap_remove(i))
    }
}
