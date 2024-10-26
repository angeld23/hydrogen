use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct IndexedContainer<T> {
    pub items: Vec<T>,
    pub indices: Vec<u32>,
}

impl<T> Default for IndexedContainer<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            indices: Default::default(),
        }
    }
}

impl<T> IndexedContainer<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(item_capacity: usize, index_capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(item_capacity),
            indices: Vec::with_capacity(index_capacity),
        }
    }

    pub fn push(&mut self, item: T) {
        self.indices.push(self.items.len() as u32);
        self.items.push(item);
    }

    pub fn push_repeated(&mut self, item: T, amount: u32) {
        self.indices.resize(
            self.indices.len() + amount as usize,
            self.items.len() as u32,
        );
        self.items.push(item);
    }

    pub fn push_relative_indexed(
        &mut self,
        items: impl IntoIterator<Item = T>,
        relative_indices: impl IntoIterator<Item = u32>,
    ) {
        let index_offset = self.items.len() as u32;
        self.indices.extend(
            relative_indices
                .into_iter()
                .map(|index| index + index_offset),
        );
        self.items.extend(items);
    }

    pub fn push_container(&mut self, other_container: Self) {
        self.items.reserve(other_container.items.len());
        self.indices.reserve(other_container.indices.len());
        let index_offset = self.items.len() as u32;
        for item in other_container.items {
            self.items.push(item);
        }
        for index in other_container.indices {
            self.indices.push(index + index_offset);
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.indices.clear();
    }
}
