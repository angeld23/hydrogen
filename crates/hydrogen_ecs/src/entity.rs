use derive_more::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, From, Into, Add, AddAssign, Sub, SubAssign)]
pub struct EntityId(pub u32);

impl From<usize> for EntityId {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}
