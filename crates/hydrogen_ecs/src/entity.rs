use derive_more::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    From,
    Into,
    Add,
    AddAssign,
    Sub,
    SubAssign,
)]
pub struct EntityId(pub u32);

impl From<usize> for EntityId {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}
