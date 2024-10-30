use crate::entity::EntityId;
use derive_more::*;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::{any::Any, collections::VecDeque, fmt, mem};

pub use hydrogen_ecs_proc_macro::{Component, SerializableComponent};

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
pub struct ComponentId(pub u64);

pub trait Component: fmt::Debug + Any + 'static {
    fn component_id(&self) -> ComponentId;
    fn display_name(&self) -> &'static str;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T> AsAny for T
where
    T: Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait DynPartialEq: AsAny {
    fn dyn_eq(&self, other: &dyn Any) -> bool;
}

impl<T> DynPartialEq for T
where
    T: PartialEq<T> + 'static,
{
    fn dyn_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<T>().map_or(false, |item| self == item)
    }
}

impl PartialEq for Box<dyn DynPartialEq> {
    fn eq(&self, other: &Self) -> bool {
        (**self).dyn_eq((**other).as_any())
    }
}

#[typetag::serde(tag = "type")]
pub trait SerializableComponent: Component + DynClone + DynPartialEq {}
dyn_clone::clone_trait_object!(SerializableComponent);

impl PartialEq for Box<dyn SerializableComponent> {
    fn eq(&self, other: &Self) -> bool {
        (**self).dyn_eq((**other).as_any())
    }
}

#[derive(Debug)]
pub struct ComponentSet {
    component_id: ComponentId,
    components: Vec<Option<Box<dyn Component>>>,
    entity_component_indices: Vec<Option<usize>>,
    deleted_component_indices: VecDeque<usize>,
}

impl ComponentSet {
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            component_id,
            components: vec![],
            entity_component_indices: vec![],
            deleted_component_indices: VecDeque::new(),
        }
    }

    pub fn has_entity(&self, entity_id: EntityId) -> bool {
        let index = entity_id.0 as usize;

        matches!(self.entity_component_indices.get(index), Some(Some(_)))
    }

    pub fn entity_component_indices(&self) -> &Vec<Option<usize>> {
        &self.entity_component_indices
    }

    fn reserve_entity_component_indices(&mut self, highest_id: usize) {
        if self.entity_component_indices.len() <= highest_id {
            self.entity_component_indices.resize(highest_id + 1, None);
        }
    }

    pub fn get(&self, entity_id: EntityId) -> Option<&Box<dyn Component>> {
        let index = entity_id.0 as usize;

        let component_index = self.entity_component_indices.get(index)?.to_owned()?;
        return self.components.get(component_index)?.as_ref();
    }

    pub fn get_mut(&mut self, entity_id: EntityId) -> Option<&mut Box<dyn Component>> {
        let index = entity_id.0 as usize;

        let component_index = self.entity_component_indices.get(index)?.to_owned()?;
        return self.components.get_mut(component_index)?.as_mut();
    }

    pub fn set(
        &mut self,
        entity_id: EntityId,
        entry: Box<dyn Component>,
    ) -> Option<Box<dyn Component>> {
        let index = entity_id.0 as usize;

        assert!(
            entry.component_id() == self.component_id,
            "ComponentId mismatch: Expected {}, got {} ({})",
            self.component_id.0,
            entry.component_id().0,
            entry.display_name()
        );

        if let Some(old_entry) = self.get_mut(entity_id) {
            return Some(mem::replace(old_entry, entry));
        }

        self.reserve_entity_component_indices(index);

        if let Some(component_index) = self.deleted_component_indices.pop_front() {
            self.components[component_index] = Some(entry);
        } else {
            self.components.push(Some(entry));
        };

        None
    }

    pub fn delete(&mut self, entity_id: EntityId) -> Option<Box<dyn Component>> {
        let index = entity_id.0 as usize;

        let component_index = self.entity_component_indices.get(index)?.to_owned()?;
        self.deleted_component_indices.push_back(component_index);
        self.entity_component_indices[index] = None;

        self.components.get_mut(component_index)?.take()
    }
}
