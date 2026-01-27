use crate::{axis::Axis, sign::Sign};
use cgmath::{Vector3, num_traits::Signed, vec3};
use serde::{Deserialize, Serialize};

/// One of six perpendicular directions.
#[derive(Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct Direction {
    pub sign: Sign,
    pub axis: Axis,
}

impl std::fmt::Debug for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match self.sign {
                Sign::Positive => "+",
                Sign::Negative => "-",
            },
            match self.axis {
                Axis::X => "X",
                Axis::Y => "Y",
                Axis::Z => "Z",
            }
        )
    }
}

impl Direction {
    /// Positive [Direction]s in a Vector3.
    ///
    /// # Example
    /// ```
    /// let positive_y = Direction::POS.y;
    /// assert_eq!(positive_y.normal(), vec3(0.0, 1.0, 0.0));
    /// ```
    pub const POS: Vector3<Self> = vec3(
        Self {
            sign: Sign::Positive,
            axis: Axis::X,
        },
        Self {
            sign: Sign::Positive,
            axis: Axis::Y,
        },
        Self {
            sign: Sign::Positive,
            axis: Axis::Z,
        },
    );
    /// Negative [Direction]s in a Vector3.
    ///
    /// # Example
    /// ```
    /// let negative_y = Direction::NEG.y;
    /// assert_eq!(negative_y.normal(), vec3(0.0, -1.0, 0.0));
    /// ```
    pub const NEG: Vector3<Self> = vec3(
        Self {
            sign: Sign::Negative,
            axis: Axis::X,
        },
        Self {
            sign: Sign::Negative,
            axis: Axis::Y,
        },
        Self {
            sign: Sign::Negative,
            axis: Axis::Z,
        },
    );

    /// All 6 directions.
    pub const ALL: [Self; 6] = [
        Self::POS.x,
        Self::NEG.x,
        Self::POS.y,
        Self::NEG.y,
        Self::POS.z,
        Self::NEG.z,
    ];

    /// North, defined as negative Z.
    pub const NORTH: Self = Self::NEG.z;
    /// East, based on north being defined as negative Z.
    pub const EAST: Self = Self::POS.x;
    /// South, based on north being defined as negative Z.
    pub const SOUTH: Self = Self::POS.z;
    /// West, based on north being defined as negative Z.
    pub const WEST: Self = Self::NEG.x;

    /// The four cardinal directions, with north defined as negative Z.
    pub const CARDINAL: [Self; 4] = [Self::NORTH, Self::EAST, Self::SOUTH, Self::WEST];

    /// Returns the closest [Direction] to a given [Vector3]'s direction.
    pub fn from_vector<T>(vector: Vector3<T>) -> Self
    where
        T: Signed + PartialOrd + Copy,
    {
        let mut biggest = (Axis::default(), T::zero());
        for axis in Axis::ALL {
            let value = axis.get_component(vector);
            if value.abs() > biggest.1.abs() {
                biggest = (axis, value);
            }
        }

        Self {
            sign: Sign::of(biggest.1),
            axis: biggest.0,
        }
    }

    /// Returns a unit [Vector3] facing this [Direction].
    pub fn normal<T>(self) -> Vector3<T>
    where
        T: Signed + Default,
    {
        self.axis.vector_with_component(self.sign.signum())
    }
}

impl std::ops::Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            sign: -self.sign,
            ..self
        }
    }
}
