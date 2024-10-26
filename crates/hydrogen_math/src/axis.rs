use cgmath::{vec2, vec3, Vector2, Vector3};
use derive_more::*;
use serde::{Deserialize, Serialize};

/// One of the 3-dimensional axes (X, Y, or Z).
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, IsVariant)]
pub enum Axis {
    #[default]
    X,
    Y,
    Z,
}

impl Axis {
    pub const ALL: [Self; 3] = [Axis::X, Axis::Y, Axis::Z];

    /// Returns a [Vector3]'s value for this axis.
    ///
    /// # Example
    /// ```
    /// let vector = vec3(2, -4, 7);
    /// assert_eq!(Axis::Z.get_component(vector), 7);
    /// ```
    pub fn get_component<T>(self, vector: Vector3<T>) -> T {
        match self {
            Self::X => vector.x,
            Self::Y => vector.y,
            Self::Z => vector.z,
        }
    }

    /// Returns an immutable reference to a [Vector3]'s value for this axis.
    ///
    /// # Example
    /// ```
    /// let vector = vec3(vec![2], vec![-4], vec![7]); // using a non-Copy component type
    /// let z_value: &Vec<i32> = Axis::Z.get_component(&vector);
    /// assert_eq!(z_value, vec![7]);
    /// ```
    pub fn get_component_ref<T>(self, vector: &Vector3<T>) -> &T {
        match self {
            Self::X => &vector.x,
            Self::Y => &vector.y,
            Self::Z => &vector.z,
        }
    }

    /// Returns a mutable reference to a [Vector3]'s value for this axis.
    ///
    /// # Example
    /// ```
    /// let mut vector = vec3(2, -4, 7);
    /// *Axis::Z.get_component_mut(&mut vector) = 45;
    /// assert_eq!(vector, vec3(2, -4, 45));
    /// ```
    pub fn get_component_mut<T>(self, vector: &mut Vector3<T>) -> &mut T {
        match self {
            Self::X => &mut vector.x,
            Self::Y => &mut vector.y,
            Self::Z => &mut vector.z,
        }
    }

    /// Updates a [Vector3]'s value for this axis.
    ///
    /// # Example
    /// ```
    /// let mut vector = vec3(2, -4, 7);
    /// Axis::Z.set_component(&mut vector, 45);
    /// assert_eq!(vector, vec3(2, -4, 45));
    /// ```
    pub fn set_component<T>(self, vector: &mut Vector3<T>, value: T) {
        match self {
            Self::X => vector.x = value,
            Self::Y => vector.y = value,
            Self::Z => vector.z = value,
        }
    }

    /// Creates a [Vector3<T>] with this axis set to a given value and the other two
    /// axes set to [T::default()](Default::default()).
    ///
    /// # Example
    /// ```
    /// let vector = Axis::Z.vector_with_component(33);
    /// assert_eq!(vector, vec3(0, 0, 33));
    /// ```
    pub fn vector_with_component<T>(self, value: T) -> Vector3<T>
    where
        T: Default,
    {
        match self {
            Self::X => vec3(value, Default::default(), Default::default()),
            Self::Y => vec3(Default::default(), value, Default::default()),
            Self::Z => vec3(Default::default(), Default::default(), value),
        }
    }

    /// X = 0, Y = 1, Z = 2.
    pub fn index(self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }

    /// Returns a [Vector2<T>] that excludes the value for this axis.
    ///
    /// # Example
    /// ```
    /// let vector = vec3(2, -4, 7);
    /// assert_eq!(Axis::Y.remove(vector), vec2(2, 7));
    /// ```
    pub fn remove<T>(self, vector: Vector3<T>) -> Vector2<T> {
        match self {
            Axis::X => vec2(vector.y, vector.z),
            Axis::Y => vec2(vector.x, vector.z),
            Axis::Z => vec2(vector.x, vector.y),
        }
    }
}
