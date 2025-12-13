//! A way to define a structure of key/value pairs with an indexed, finite amount of possible states.
//!
//! Each possible permutation of a given finite state definition is assigned to an index. This allows
//! an object's properties to be stored as a single integer.
//!
//! TODO: This whole system is kinda janky, especially with serialization of StringEnum. I should probably rework it.
//! TODO: nuh uh

use derive_more::*;
use linear_map::set::LinearSet;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

type FStateMap<T> = BTreeMap<&'static str, T>;

/// An index for one possible permutation of an FProperty.
pub type FPropertyVariantIndex = u32;
/// An index for one possible permutation of an FState.
pub type FStateVariantIndex = u32;

/// The type constraint of an FProperty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IsVariant)]
pub enum FPropertyKind {
    /// `true` or `false`.
    Boolean,
    /// A signed integer ranging from `start` to `end` (inclusive).
    /// `end` is not required to be larger than `start`.
    Integer { start: i32, end: i32 },
    /// Like an enum, but with static [string slices](str) as the possible values.
    ///
    /// Note: May not be the best solution.
    StringEnum(&'static [&'static str]),
}

/// The value of an FProperty. See [FPropertyKind] for documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Unwrap, IsVariant, From)]
pub enum FPropertyValue {
    Boolean(bool),
    Integer(i32),
    StringEnum(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash, Unwrap, IsVariant, From)]
pub enum FPropertyOwnedValue {
    Boolean(bool),
    Integer(i32),
    StringEnum(String),
}

impl From<FPropertyValue> for FPropertyOwnedValue {
    fn from(value: FPropertyValue) -> Self {
        match value {
            FPropertyValue::Boolean(b) => Self::Boolean(b),
            FPropertyValue::Integer(i) => Self::Integer(i),
            FPropertyValue::StringEnum(s) => Self::StringEnum(s.to_owned()),
        }
    }
}

#[derive(Debug, Error)]
pub enum FPropertyError {
    #[error("Property not found")]
    NotFound,
    #[error("Property kind mismatch")]
    KindMismatch,
    #[error("Invalid value for this property")]
    InvalidValue,
}

impl FPropertyKind {
    /// The amount of possible unique values for this [FPropertyKind].
    pub fn variant_count(self) -> FPropertyVariantIndex {
        match self {
            Self::Boolean => 2,
            Self::Integer { start, end } => (start - end).unsigned_abs() + 1,
            Self::StringEnum(variants) => variants.len().max(1) as FPropertyVariantIndex,
        }
    }

    /// Generates the value assigned to the given variant index for this [FPropertyKind],
    /// as long as it's within range.
    pub fn get_variant_value(self, index: FPropertyVariantIndex) -> Option<FPropertyValue> {
        Some(match self {
            Self::Boolean => FPropertyValue::Boolean(match index {
                0 => false,
                1 => true,
                _ => return None,
            }),
            Self::Integer { start, end } => FPropertyValue::Integer({
                if index as i32 > (end - start).abs() {
                    return None;
                }

                start.min(end) + index as i32
            }),
            Self::StringEnum(variants) => FPropertyValue::StringEnum(if variants.is_empty() {
                // the enum shouldn't be empty but we can't have a property with zero variants so
                ""
            } else {
                variants.get(index as usize)?
            }),
        })
    }

    /// Given an FPropertyValue, returns this [FPropertyKind]'s variant index for that value.
    ///
    /// # Errors
    /// [KindMismatch](FPropertyError::KindMismatch): The type of the value does not match this [FPropertyKind].
    ///
    /// [InvalidValue](FPropertyError::InvalidValue): The value is of the right type, but it is not in the
    /// range of possible variants for this particular [FPropertyKind]. (e.g. an integer that is out of range)
    pub fn get_variant_index(
        self,
        value: FPropertyValue,
    ) -> Result<FPropertyVariantIndex, FPropertyError> {
        Ok(match (self, value) {
            (Self::Boolean, FPropertyValue::Boolean(value)) => {
                if value {
                    1
                } else {
                    0
                }
            }
            (Self::Integer { start, end }, FPropertyValue::Integer(value)) => {
                // this is needed because start can be larger than end
                let min = start.min(end);
                let max = start.max(end);
                if value < min || value > max {
                    return Err(FPropertyError::InvalidValue);
                }

                (value - min) as FPropertyVariantIndex
            }
            (Self::StringEnum(variants), FPropertyValue::StringEnum(value)) => {
                for (i, &variant) in variants.iter().enumerate() {
                    if variant == value {
                        return Ok(i as FPropertyVariantIndex);
                    }
                }

                return Err(FPropertyError::InvalidValue);
            }
            _ => return Err(FPropertyError::KindMismatch),
        })
    }

    pub fn get_owned_variant_index(
        self,
        value: &FPropertyOwnedValue,
    ) -> Result<FPropertyVariantIndex, FPropertyError> {
        match (self, value) {
            (_, FPropertyOwnedValue::Boolean(b)) => {
                self.get_variant_index(FPropertyValue::Boolean(*b))
            }
            (_, FPropertyOwnedValue::Integer(i)) => {
                self.get_variant_index(FPropertyValue::Integer(*i))
            }
            (FPropertyKind::StringEnum(variants), FPropertyOwnedValue::StringEnum(value)) => {
                for (i, &variant) in variants.iter().enumerate() {
                    if variant == value {
                        return Ok(i as FPropertyVariantIndex);
                    }
                }

                Err(FPropertyError::InvalidValue)
            }
            _ => Err(FPropertyError::KindMismatch),
        }
    }
}

/// A builder used to construct an [FStateDefinition].
///
/// # Example
/// ```
/// let state_definition = FStateDefinitionBuilder::new()
///     .boolean("active")
///     .integer("count", 0, 3)
///     .string_enum("side", &["left", "right", "top", "bottom"])
///     .build();
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FStateDefinitionBuilder(FStateMap<FPropertyKind>);

impl FStateDefinitionBuilder {
    /// Creates a new [FStateDefinitionBuilder]. After adding the desired properties, finish it with `build()`.
    pub fn new() -> Self {
        Default::default()
    }

    pub fn merge(mut self, other: FStateDefinitionBuilder) -> Self {
        for (key, kind) in other.0 {
            self.0.insert(key, kind);
        }
        self
    }

    fn property(mut self, key: &'static str, kind: FPropertyKind) -> Self {
        self.0.insert(key, kind);
        self
    }

    /// Append an [FPropertyKind::Boolean] with a given key.
    pub fn boolean(self, key: &'static str) -> Self {
        self.property(key, FPropertyKind::Boolean)
    }

    /// Append an [FPropertyKind::Integer] with a given key and range.
    pub fn integer(self, key: &'static str, start: i32, end: i32) -> Self {
        self.property(key, FPropertyKind::Integer { start, end })
    }

    /// Append an [FPropertyKind::StringEnum] with a given key and list of variants.
    ///
    /// # Panics
    /// Panics if `variants` contains any duplicates.
    pub fn string_enum(self, key: &'static str, variants: &'static [&'static str]) -> Self {
        // runtime check to ensure uniqueness
        let mut occurred_values = LinearSet::with_capacity(variants.len());
        for &variant in variants.iter() {
            if !occurred_values.insert(variant) {
                panic!(
                    "Duplicate string enum variant for property '{}': {}",
                    key, variant
                );
            }
        }

        self.property(key, FPropertyKind::StringEnum(variants))
    }

    /// Creates an [FStateDefinition] with all previously defined properties.
    pub fn build(self) -> FStateDefinition {
        let mut properties = FStateMap::<FPropertyDefinition>::new();

        let mut place_value = 1;
        for (key, kind) in self.0.into_iter() {
            let variant_count = kind.variant_count();
            properties.insert(
                key,
                FPropertyDefinition {
                    kind,
                    variant_count,
                    place_value,
                },
            );
            place_value *= variant_count;
        }

        FStateDefinition(properties)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FPropertyDefinition {
    kind: FPropertyKind,
    variant_count: FPropertyVariantIndex,
    place_value: FStateVariantIndex,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct FStateDefinition(FStateMap<FPropertyDefinition>);

impl FStateDefinition {
    /// Creates an [FStateDefinitionBuilder], not an [FStateDefinition] directly.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> FStateDefinitionBuilder {
        FStateDefinitionBuilder::default()
    }

    /// The amount of unique possible values for this [FStateDefinition].
    pub fn variant_count(&self) -> FStateVariantIndex {
        self.0
            .iter()
            .fold(1, |accumulator, (_, property_definition)| {
                accumulator * property_definition.variant_count
            })
    }

    /// Generates the [FState] assigned to the given variant index for this [FStateDefinition],
    /// as long as it's within range.
    pub fn get_variant_state(&self, index: FStateVariantIndex) -> Option<FState> {
        if index >= self.variant_count() {
            return None;
        }

        let mut properties = FStateMap::<(FPropertyValue, FPropertyDefinition)>::new();

        let mut factor = 1;
        self.0.iter().for_each(|(key, property_definition)| {
            let radix = property_definition.variant_count;
            let variant_index = index / factor % radix;

            properties.insert(
                key,
                (
                    property_definition
                        .kind
                        .get_variant_value(variant_index)
                        .unwrap(),
                    *property_definition,
                ),
            );

            factor *= radix;
        });

        Some(FState {
            properties,
            variant_index: index,
        })
    }

    pub fn state_from_portable(&self, values: &BTreeMap<String, FPropertyOwnedValue>) -> FState {
        let mut state = self.get_variant_state(0).unwrap();

        for (key, value) in values {
            if let Some(propery_definition) = self.0.get(key.as_str()) {
                // convert owned value to borrowed value
                if let Ok(property_variant_index) =
                    propery_definition.kind.get_owned_variant_index(value)
                    && let Some(borrowed_value) = propery_definition
                        .kind
                        .get_variant_value(property_variant_index)
                    // just keep creating a new state for setting each property using variant_index_after_property_set
                    // kinda hacky but whatever
                    && let Ok(new_variant_index) =
                        state.variant_index_after_property_set(key, borrowed_value)
                        && let Some(new_state) = self.get_variant_state(new_variant_index)
                {
                    state = new_state;
                }
            }
        }

        state
    }
}

/// A collection of [FPropertyValue]s, produced as a variant of an [FStateDefinition].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FState {
    properties: FStateMap<(FPropertyValue, FPropertyDefinition)>,
    variant_index: FStateVariantIndex,
}

macro_rules! property_getter {
    ($fn_name:ident, $return_type:ty, $property_variant:ident) => {
        /// Retrieves the value of the property with a given key.
        ///
        /// # Errors
        /// [KindMismatch](FPropertyError::KindMismatch): The property's type does not match the
        /// type that this method is meant to extract.
        ///
        /// [NotFound](FPropertyError::NotFound): No property with the provided key exists.
        pub fn $fn_name(&self, key: &str) -> Result<$return_type, FPropertyError> {
            match self.get_property(key) {
                Some((FPropertyValue::$property_variant(value), _)) => Ok(value),
                Some(_) => Err(FPropertyError::KindMismatch),
                None => Err(FPropertyError::NotFound),
            }
        }
    };
}

impl FState {
    fn get_property(&self, key: &str) -> Option<(FPropertyValue, FPropertyDefinition)> {
        self.properties.get(key).copied()
    }

    property_getter!(get_boolean, bool, Boolean);
    property_getter!(get_integer, i32, Integer);
    property_getter!(get_string_enum, &'static str, StringEnum);

    pub fn variant_index(&self) -> FStateVariantIndex {
        self.variant_index
    }

    /// Calculates the variant index of an [FState] equivalent to changing a given property
    /// to a provided value.
    ///
    /// Note that does not actually mutate the state, it merely provides a variant index that
    /// can be passed into [`FStateDefinition::get_variant_state()`] in order to retrieve a new [FState].
    ///
    /// # Errors
    /// This method will fail if the property does not exist, the value's type does not match, or
    /// the value is out of range.
    pub fn variant_index_after_property_set(
        &self,
        key: &str,
        new_value: FPropertyValue,
    ) -> Result<FStateVariantIndex, FPropertyError> {
        let (old_value, property_definition) =
            self.get_property(key).ok_or(FPropertyError::NotFound)?;

        let new_variant_index = property_definition.kind.get_variant_index(new_value)?;
        let old_variant_index = property_definition.kind.get_variant_index(old_value)?;

        let index_diff = new_variant_index as i32 - old_variant_index as i32;

        Ok(
            (self.variant_index as i32 + index_diff * property_definition.place_value as i32)
                as FStateVariantIndex,
        )
    }

    pub fn to_portable(&self) -> BTreeMap<String, FPropertyOwnedValue> {
        self.properties
            .iter()
            .map(|(&key, (value, _))| (key.to_owned(), (*value).into()))
            .collect()
    }
}
