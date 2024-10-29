use std::collections::BTreeSet;

use derive_more::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, IsVariant, Unwrap, TryUnwrap)]
pub enum Selection<T: Ord> {
    Whitelist(BTreeSet<T>),
    Blacklist(BTreeSet<T>),
}

impl<T: Ord> Selection<T> {
    pub fn none() -> Self {
        Self::Whitelist(BTreeSet::new())
    }

    pub fn all() -> Self {
        Self::Blacklist(BTreeSet::new())
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

    pub fn get_values(&self) -> &BTreeSet<T> {
        match self {
            Selection::Whitelist(values) => values,
            Selection::Blacklist(values) => values,
        }
    }

    pub fn get_values_mut(&mut self) -> &mut BTreeSet<T> {
        match self {
            Selection::Whitelist(values) => values,
            Selection::Blacklist(values) => values,
        }
    }
}
