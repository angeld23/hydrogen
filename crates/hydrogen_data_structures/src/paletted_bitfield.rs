use linear_map::set::LinearSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PalettedBitfield<T> {
    data: Vec<u64>,
    bit_width: usize,
    palette: Vec<T>,
    length: usize,
}

/// Calculates the minimum required amount of bits to store a given amount of unique values.
///
/// Note: A unique item count of 1 will have a requirement of 0 bits, since the only required
/// information would be the length of the container.
fn get_required_bits(unique_items: usize) -> usize {
    if unique_items == 0 {
        0
    } else {
        (usize::BITS - (unique_items - 1).leading_zeros()) as usize
    }
}

/// Extracts a value from a bitfield.
///
/// Returns (container_index, bit_index, container, item).
fn bitfield_extract(
    data: &[u64],
    bit_width: usize,
    item_index: usize,
) -> (usize, usize, u128, usize) {
    if bit_width == 0 {
        return (0, 0, 0, 0);
    }

    // a "container" is two adjacient u64s from the bitfield combined into a single u128
    // this is done because values can sometimes span two u64s
    let container_index = item_index * bit_width / 64;
    let bit_index = item_index * bit_width % 64;
    // the u64 that contains the first bit of the value is the significant half of the u128
    let container = ((data[container_index] as u128) << 64)
        | *data.get(container_index + 1).unwrap_or(&0) as u128;

    // shift the value such that its rightmost bit is now the rightmost bit of the entire container
    let shifted = container >> (128 - bit_index - bit_width);
    // 1, repeated bit_width times from the right
    let mask = (1 << bit_width) - 1;
    // everything to the left of the value is zeroed out, so only the value itself remains
    let item = (shifted & mask) as usize;

    (container_index, bit_index, container, item)
}

/// Inserts a value into a bitfield.
fn bitfield_insert(data: &mut [u64], bit_width: usize, item_index: usize, value: usize) {
    let (container_index, bit_index, mut container, _) =
        bitfield_extract(data, bit_width, item_index);

    let shift_amount = 128 - bit_index - bit_width;
    // apply a bitmask of all 1s, except for the bits in which the new value will be
    // inserted (in order to remove the old value)
    container &= !(((1 << bit_width) - 1) << shift_amount);
    // insert the new one
    container |= (value as u128) << shift_amount;

    // insert the container back in
    data[container_index] = (container >> 64) as u64;
    if container_index + 1 < data.len() {
        data[container_index + 1] = (container & u64::MAX as u128) as u64;
    }
}

impl<T> Default for PalettedBitfield<T> {
    /// Creates an empty PalettedBitfield.
    fn default() -> Self {
        Self {
            data: vec![],
            bit_width: 0,
            palette: vec![],
            length: 0,
        }
    }
}

/// A container that is implemented using a palette and a list of indices compactly stored with a bitfield.
impl<T> PalettedBitfield<T>
where
    T: Clone + Eq,
{
    /// Creates an empty PalettedBitfield.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates an empty PalettedBitfield with a given bit width. This method can be used to avoid
    /// unneeded index resizing.
    pub fn with_bit_width(bit_width: usize) -> Self {
        if bit_width > 64 {
            panic!("Bit width of {bit_width} is higher than the maximum of 64");
        }

        Self {
            data: vec![],
            bit_width,
            palette: vec![],
            length: 0,
        }
    }

    /// Creates a PalettedBitfield and fills it with the contents of a slice.
    pub fn with_items(items: &[T]) -> Self {
        let mut unique_items = LinearSet::<&T>::with_capacity(32);
        for item in items.iter() {
            unique_items.insert(item);
        }

        let mut field = Self {
            data: vec![0; items.len().div_ceil(64)],
            bit_width: get_required_bits(unique_items.len()),
            palette: vec![],
            length: items.len(),
        };

        for (index, item) in items.iter().enumerate() {
            field.set(index, item);
        }

        field
    }

    /// The amount of items stored in this PalettedBitfield.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Whether this PalettedBitfield's length is zero.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Changes the bit width of the indices. This is obviously expensive as the entire bitfield
    /// needs to be recreated from scratch.
    fn resize_bit_width(&mut self, new_bit_width: usize) {
        if new_bit_width == self.bit_width {
            return;
        }
        if new_bit_width == 0 {
            self.data = vec![];
            self.bit_width = 0;
            return;
        }

        let old_bit_width = self.bit_width;
        self.bit_width = new_bit_width;

        // swap in a new properly-sized bitfield
        let old_data = std::mem::replace(
            &mut self.data,
            vec![0u64; (self.length * self.bit_width).div_ceil(64)],
        );

        // migrate the indices from the old bitfield into the new one
        for index in 0..self.length {
            let (_, _, _, palette_index) = bitfield_extract(&old_data, old_bit_width, index);
            bitfield_insert(&mut self.data, new_bit_width, index, palette_index);
        }
    }

    /// Calls resize_bit_width if needed.
    fn check_size(&mut self) {
        let required_bits = get_required_bits(self.palette.len());

        if required_bits != self.bit_width {
            self.resize_bit_width(required_bits);
        }
    }

    /// Retrieves the palette index of an item if it exists.
    fn get_pallete_index(&self, item: &T) -> Option<usize> {
        for (index, other) in self.palette.iter().enumerate() {
            if item == other {
                return Some(index);
            }
        }
        None
    }

    /// Retrieves the palette index of an item, and adds it to the palette if it does not exist.
    fn get_or_add_pallete_index(&mut self, item: &T) -> usize {
        self.get_pallete_index(item).unwrap_or_else(|| {
            self.palette.push(item.clone());
            self.check_size();
            self.palette.len() - 1
        })
    }

    /// Retrieves a reference to the item located at a given index, if that index exists.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.length {
            return None;
        }

        if self.bit_width == 0 {
            return self.palette.first();
        }

        let (_, _, _, palette_index) = bitfield_extract(&self.data, self.bit_width, index);

        self.palette.get(palette_index)
    }

    /// Inserts a value into the given index, if that index exists.
    ///
    /// Returns whether the value was successfully inserted.
    pub fn set(&mut self, index: usize, item: &T) -> bool {
        if index >= self.length {
            return false;
        }

        let palette_index = self.get_or_add_pallete_index(item);
        if self.bit_width == 0 {
            return true;
        }
        bitfield_insert(&mut self.data, self.bit_width, index, palette_index);

        true
    }

    /// Resizes the PalettedBitfield to a new length.
    ///
    /// If the new length is larger than the current one, all new indices are
    /// initialized to point to the first value that was originally inserted
    /// into this PalettedBitfield.
    pub fn resize(&mut self, new_length: usize) {
        self.data
            .resize((new_length * self.bit_width).div_ceil(64), 0);
        self.length = new_length;
    }

    /// Pushes an item to the end of the PalettedBitfield.
    pub fn push(&mut self, item: &T) {
        self.resize(self.length + 1);
        self.set(self.length - 1, item);
    }

    /// Removes the item at the end of the PalettedBitfield and returns it,
    /// as long as it isn't empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            return None;
        }

        let item = self.get(self.length - 1).cloned();
        self.resize(self.length - 1);
        item
    }

    pub fn unpack(&self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len());
        for index in 0..self.len() {
            result.push(self.get(index).unwrap().clone());
        }
        result
    }
}
