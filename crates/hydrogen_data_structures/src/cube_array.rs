use cgmath::{vec3, Vector3};
use hydrogen_math::direction::Direction;

pub fn vec_to_sized_box<T, const S: usize>(vec: Vec<T>) -> Option<Box<[T; S]>> {
    if vec.len() == S {
        Some(
            // evil typecast to include the size in the boxed slice type
            //
            // a previous version of this was causing some hair-pulling random stack overflows due to
            // heap corruption. heed my warnings and either avoid unsafe bullshit at all costs, or
            // always remember that if you ever get weird, inconsistent low-level errors, blame
            // an unsafe block first and foremost, then blame multithreading, THEN blame yourself.
            //
            // SAFETY:
            // the vec's length is ensured to be exactly equal to S
            unsafe {
                Box::from_raw(std::boxed::Box::into_raw(vec.into_boxed_slice()) as *mut [T; S])
            },
        )
    } else {
        None
    }
}

/// A 3-dimensional, heap-allocated array of equal width, height, and length.
///
/// The actual array is flat. Positions are converted into indices.
#[derive(Debug, Clone)]
pub struct CubeArray<const SIDE_LENGTH: i32, T>
where
    [(); SIDE_LENGTH.pow(3) as usize]:,
{
    pub items: Box<[T; SIDE_LENGTH.pow(3) as usize]>,
}

impl<const SIDE_LENGTH: i32, T> Default for CubeArray<SIDE_LENGTH, T>
where
    [(); SIDE_LENGTH.pow(3) as usize]:,
    T: Default,
{
    /// Create a [CubeArray] with all items initialized to `T::default()`.
    fn default() -> Self {
        Self {
            items: vec_to_sized_box(Vec::from_iter(
                std::iter::repeat_with(|| Default::default()).take(SIDE_LENGTH.pow(3) as usize),
            ))
            .unwrap(),
        }
    }
}

impl<const SIDE_LENGTH: i32, T> CubeArray<SIDE_LENGTH, T>
where
    [(); SIDE_LENGTH.pow(3) as usize]:,
{
    /// Create a [CubeArray] with all items initialized to `T::default()`.
    pub fn new() -> Self
    where
        T: Default,
    {
        Default::default()
    }

    pub fn cloned(value: T) -> Self
    where
        T: Clone,
    {
        Self {
            items: vec_to_sized_box(vec![value; SIDE_LENGTH.pow(3) as usize]).unwrap(),
        }
    }

    /// Converts a position to an index, as long as the position is within range.
    pub fn get_index(position: Vector3<i32>) -> Option<usize> {
        for axis in [position.x, position.y, position.z] {
            if axis < 0 || axis >= SIDE_LENGTH {
                return None;
            }
        }

        Some(
            (position.x % SIDE_LENGTH + position.y * SIDE_LENGTH + position.z * SIDE_LENGTH.pow(2))
                as usize,
        )
    }

    /// Converts an index to a position, as long as the index is within range.
    pub fn get_position(index: usize) -> Option<Vector3<i32>> {
        let index = index as i32;

        if index >= SIDE_LENGTH.pow(3) {
            return None;
        }

        Some(vec3(
            index % SIDE_LENGTH,
            index / SIDE_LENGTH % SIDE_LENGTH,
            index / SIDE_LENGTH.pow(2),
        ))
    }

    /// Retrieves a reference to the item located at a given position in the array.
    ///
    /// # Panics
    /// This method panics if the index is out of range. Use `CubeArray::try_get` if
    /// you want the method to return [None] instead of panicking.
    pub fn get(&self, position: Vector3<i32>) -> &T {
        &self.items[Self::get_index(position).unwrap()]
    }

    /// Retrieves a reference to the item located at a given position in the array,
    /// as long as the position is within range.
    pub fn try_get(&self, position: Vector3<i32>) -> Option<&T> {
        Some(&self.items[Self::get_index(position)?])
    }

    /// Retrieves a mutable reference to the item located at a given position in the array.
    ///
    /// # Panics
    /// This method panics if the index is out of range. Use `CubeArray::try_get_mut` if
    /// you want the method to return [None] instead of panicking.
    pub fn get_mut(&mut self, position: Vector3<i32>) -> &mut T {
        &mut self.items[Self::get_index(position).unwrap()]
    }

    /// Retrieves a mutable reference to the item located at a given position in the array,
    /// as long as the position is within range.
    pub fn try_get_mut(&mut self, position: Vector3<i32>) -> Option<&mut T> {
        Some(&mut self.items[Self::get_index(position)?])
    }

    /// Sets the value located at a given position in the array and returns the old one.
    ///
    /// # Panics
    /// This method panics if the index is out of range. Use `CubeArray::try_set` if
    /// you want the method to return [None] instead of panicking.
    pub fn set(&mut self, position: Vector3<i32>, value: T) -> T {
        std::mem::replace(&mut self.items[Self::get_index(position).unwrap()], value)
    }

    /// Sets the value located at a given position in the array and returns the old one,
    /// as long as the position is within range.
    pub fn try_set(&mut self, position: Vector3<i32>, value: T) -> Option<T> {
        Self::get_index(position).map(|_| self.set(position, value))
    }

    /// Creates an iterator over up to 6 direct neighbors to a given position in the array.
    ///
    /// Each item is paired with a [Direction] to indicate which neighbor it is.
    pub fn get_neighbors(&self, position: Vector3<i32>) -> impl Iterator<Item = (Direction, &T)> {
        Direction::ALL.iter().filter_map(move |&direction| {
            Some((direction, self.try_get(position + direction.normal())?))
        })
    }
}
