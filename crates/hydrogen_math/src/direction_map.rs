use crate::{axis::Axis, direction::Direction, sign::Sign};
use cgmath::{num_traits::Signed, BaseFloat, Deg, Matrix3, Rotation3};
use serde::{Deserialize, Serialize};

/// Maps each of the six [Direction]s to a value. All [Direction]s must be mapped to a value.
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct DirectionMap<T> {
    pub pos_x: T,
    pub neg_x: T,

    pub pos_y: T,
    pub neg_y: T,

    pub pos_z: T,
    pub neg_z: T,
}

impl<T> From<T> for DirectionMap<T>
where
    T: Clone,
{
    fn from(value: T) -> Self {
        Self::uniform(&value)
    }
}

impl<T> DirectionMap<T> {
    /// All six [Direction]s mapped to the same cloned value.
    pub fn uniform(value: &T) -> Self
    where
        T: Clone,
    {
        Self {
            pos_x: value.clone(),
            neg_x: value.clone(),
            pos_y: value.clone(),
            neg_y: value.clone(),
            pos_z: value.clone(),
            neg_z: value.clone(),
        }
    }

    /// One value mapped to +Y, four mapped to +/- X/Z, and one mapped to -Y.
    pub fn top_sides_bottom(top: &T, sides: &T, bottom: &T) -> Self
    where
        T: Clone,
    {
        Self {
            pos_x: sides.clone(),
            neg_x: sides.clone(),
            pos_y: top.clone(),
            neg_y: bottom.clone(),
            pos_z: sides.clone(),
            neg_z: sides.clone(),
        }
    }

    /// Creates a map using a closure that takes a [Direction] as its only argument
    /// and returns the value to be mapped to that direction.
    ///
    /// # Example
    /// ```
    /// let map = DirectionMap::from_fn(|direction| direction.sign);
    /// assert_eq!(
    ///     map,
    ///     DirectionMap {
    ///         pos_x: Sign::Positive,
    ///         neg_x: Sign::Negative,
    ///         pos_y: Sign::Positive,
    ///         neg_y: Sign::Negative,
    ///         pos_z: Sign::Positive,
    ///         neg_z: Sign::Negative,
    ///     }
    /// );
    /// ```
    pub fn from_fn(mut function: impl FnMut(Direction) -> T) -> Self {
        Self {
            pos_x: function(Direction::POS.x),
            neg_x: function(Direction::NEG.x),
            pos_y: function(Direction::POS.y),
            neg_y: function(Direction::NEG.y),
            pos_z: function(Direction::POS.z),
            neg_z: function(Direction::NEG.z),
        }
    }

    /// Retrieve an immutable reference to the value mapped to the given direction.
    pub fn get(&self, direction: Direction) -> &T {
        match direction.sign {
            Sign::Positive => match direction.axis {
                Axis::X => &self.pos_x,
                Axis::Y => &self.pos_y,
                Axis::Z => &self.pos_z,
            },
            Sign::Negative => match direction.axis {
                Axis::X => &self.neg_x,
                Axis::Y => &self.neg_y,
                Axis::Z => &self.neg_z,
            },
        }
    }

    /// Retrieve a mutable reference to the value mapped to the given direction.
    pub fn get_mut(&mut self, direction: Direction) -> &mut T {
        match direction.sign {
            Sign::Positive => match direction.axis {
                Axis::X => &mut self.pos_x,
                Axis::Y => &mut self.pos_y,
                Axis::Z => &mut self.pos_z,
            },
            Sign::Negative => match direction.axis {
                Axis::X => &mut self.neg_x,
                Axis::Y => &mut self.neg_y,
                Axis::Z => &mut self.neg_z,
            },
        }
    }

    /// Update the value currently mapped to the given direction. Returns the previous value.
    pub fn set(&mut self, direction: Direction, value: T) -> T {
        std::mem::replace(self.get_mut(direction), value)
    }

    /// "Rotates" the map using a transform.
    pub fn rotate<F: BaseFloat + Signed + Default>(&mut self, transform: impl Rotation3<Scalar = F>)
    where
        T: Default,
    {
        let old_self = std::mem::take(self);

        for (direction, value) in old_self.into_iter() {
            let transformed_normal = transform.rotate_vector(direction.normal::<F>());
            self.set(Direction::from_vector(transformed_normal), value);
        }
    }

    /// "Rotates" the map using a rotation matrix.
    pub fn rotate_with_matrix<F: BaseFloat + Signed + Default>(&mut self, matrix: Matrix3<F>)
    where
        T: Default,
    {
        let old_self = std::mem::take(self);

        for (direction, value) in old_self.into_iter() {
            let transformed_normal = matrix * direction.normal::<F>();
            self.set(Direction::from_vector(transformed_normal), value);
        }
    }

    /// Creates an iterator over immutable references to each value paired with
    /// their corresponding [Direction].
    pub fn iter(&self) -> impl Iterator<Item = (Direction, &T)> {
        Direction::ALL
            .iter()
            .map(|&direction| (direction, self.get(direction)))
    }
}

impl<T> IntoIterator for DirectionMap<T> {
    type Item = (Direction, T);

    type IntoIter = core::array::IntoIter<Self::Item, 6>;

    /// Consumes the map and creates an iterator over each value paired with
    /// its corresponding [Direction].
    fn into_iter(self) -> Self::IntoIter {
        [
            (Direction::POS.x, self.pos_x),
            (Direction::NEG.x, self.neg_x),
            (Direction::POS.y, self.pos_y),
            (Direction::NEG.y, self.neg_y),
            (Direction::POS.z, self.pos_z),
            (Direction::NEG.z, self.neg_z),
        ]
        .into_iter()
    }
}
