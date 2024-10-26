use cgmath::num_traits::Signed;
use serde::{Deserialize, Serialize};

/// Positive or Negative. Zero not included.
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum Sign {
    #[default]
    Positive,
    Negative,
}

impl Sign {
    /// Shorthand for [Sign::Positive].
    const POS: Self = Self::Positive;
    /// Shorthand for [Sign::Negative].
    const NEG: Self = Self::Negative;

    /// Returns the sign of a value. Zero is considered [Positive](Sign::Positive).
    pub fn of<T>(value: T) -> Self
    where
        T: Signed,
    {
        if value.is_negative() {
            Self::Negative
        } else {
            Self::Positive
        }
    }

    /// `T::one()` if [Positive](Sign::Positive), `-T::one()` if [Negative](Sign::Negative).
    pub fn signum<T>(self) -> T
    where
        T: Signed,
    {
        match self {
            Self::Positive => T::one(),
            Self::Negative => -T::one(),
        }
    }

    pub fn is_positive(self) -> bool {
        self == Self::Positive
    }

    pub fn is_negative(self) -> bool {
        self == Self::Negative
    }
}

impl std::ops::Neg for Sign {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Positive => Self::Negative,
            Self::Negative => Self::Positive,
        }
    }
}
