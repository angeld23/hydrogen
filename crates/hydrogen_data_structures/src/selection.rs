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
}
