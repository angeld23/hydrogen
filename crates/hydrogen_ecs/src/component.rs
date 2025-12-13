use crate::entity::EntityId;
use derive_more::*;
use dyn_clone::DynClone;
use hydrogen_core::dyn_util::DynPartialEq;
use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    array,
    collections::{BTreeMap, VecDeque},
    fmt, mem, ptr,
};

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

pub trait Component: fmt::Debug + Any + 'static + Send + Sync {
    fn component_id(&self) -> ComponentId;
    fn display_name(&self) -> &'static str;
    fn any_ref(&self) -> &dyn Any;

    fn is_serializable(&self) -> bool;
    fn as_serializable(&self) -> Option<&dyn SerializableComponent>;
    fn as_serializable_mut(&mut self) -> Option<&mut dyn SerializableComponent>;
}

impl dyn Component {
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.any_ref().downcast_ref()
    }
}

#[typetag::serde]
pub trait SerializableComponent: Component + DynClone + DynPartialEq + Send + Sync {
    fn clone_box(&self) -> Box<dyn SerializableComponent>;
}
dyn_clone::clone_trait_object!(SerializableComponent);

impl PartialEq for dyn SerializableComponent {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other.as_any())
    }
}

/// The container for every instance of a given type of component in a world.
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

    pub fn has_component_instance(&self, component: &impl Component) -> bool {
        self.components.iter().any(|c| {
            if let Some(other_component) = c {
                // check to see if the pointers match
                ptr::eq(other_component.as_ref(), component as *const dyn Component)
            } else {
                false
            }
        })
    }

    pub fn get_entity_from_component(&self, component: &impl Component) -> Option<EntityId> {
        self.entity_component_indices.iter().enumerate().find_map(
            |(entity_id, &component_index)| {
                ptr::eq(
                    self.components.get(component_index?)?.as_ref()?.as_ref(),
                    component as *const dyn Component,
                )
                .then_some(entity_id.into())
            },
        )
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
        self.components.get(component_index)?.as_ref()
    }

    pub fn get_mut(&mut self, entity_id: EntityId) -> Option<&mut Box<dyn Component>> {
        let index = entity_id.0 as usize;

        let component_index = self.entity_component_indices.get(index)?.to_owned()?;
        self.components.get_mut(component_index)?.as_mut()
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
            self.entity_component_indices[index] = Some(component_index);
        } else {
            self.components.push(Some(entry));
            self.entity_component_indices[index] = Some(self.components.len() - 1);
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

/// A container that can be used to bundle together the components of one object, ECS entity or otherwise. Contains at most *one* of
/// each Component type.
#[derive(Debug, Default)]
pub struct ComponentBundle {
    components: BTreeMap<ComponentId, Box<dyn Component>>,
}

impl ComponentBundle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_component(&self, component_id: ComponentId) -> bool {
        self.components.contains_key(&component_id)
    }

    pub fn get_component(&self, component_id: ComponentId) -> Option<&Box<dyn Component>> {
        self.components.get(&component_id)
    }

    pub fn get_component_mut(
        &mut self,
        component_id: ComponentId,
    ) -> Option<&mut Box<dyn Component>> {
        self.components.get_mut(&component_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (ComponentId, &Box<dyn Component>)> {
        self.components
            .iter()
            .map(|(&id, component)| (id, component))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (ComponentId, &mut Box<dyn Component>)> {
        self.components
            .iter_mut()
            .map(|(&id, component)| (id, component))
    }

    pub fn iter_serializable(
        &self,
    ) -> impl Iterator<Item = (ComponentId, &dyn SerializableComponent)> {
        self.iter().filter_map(|(component_id, component)| {
            Some((component_id, component.as_serializable()?))
        })
    }

    pub fn iter_serializable_mut(
        &mut self,
    ) -> impl Iterator<Item = (ComponentId, &mut dyn SerializableComponent)> {
        self.iter_mut().filter_map(|(component_id, component)| {
            Some((component_id, component.as_serializable_mut()?))
        })
    }

    pub fn set_component<T: Component>(&mut self, component: T) -> Option<T> {
        if let Some(old_component) = self
            .components
            .insert(component.component_id(), Box::new(component))
        {
            return Some(*Box::<dyn Any + 'static>::downcast::<T>(old_component).ok()?);
        }

        None
    }

    pub fn delete_component(&mut self, component_id: ComponentId) -> Option<Box<dyn Component>> {
        self.components.remove(&component_id)
    }

    pub fn query<const WITH: usize, const WITHOUT: usize>(
        &self,
        with: [ComponentId; WITH],
        without: [ComponentId; WITHOUT],
    ) -> Option<[&Box<dyn Component>; WITH]> {
        if with.is_empty() {
            return None;
        }

        for &excluded_component_id in without.iter() {
            if self.has_component(excluded_component_id) {
                return None;
            }
        }

        let mut component_slots: [Option<&Box<dyn Component>>; WITH] = array::from_fn(|_| None);
        for (index, slot) in component_slots.iter_mut().enumerate() {
            *slot = Some(self.get_component(with[index])?)
        }

        Some(array::from_fn(|index| component_slots[index].unwrap()))
    }

    pub fn query_mut<const WITH: usize, const WITHOUT: usize>(
        &mut self,
        with: [ComponentId; WITH],
        without: [ComponentId; WITHOUT],
    ) -> Option<[&mut Box<dyn Component>; WITH]> {
        if with.is_empty() {
            return None;
        }

        for &excluded_component_id in without.iter() {
            if self.has_component(excluded_component_id) {
                return None;
            }
        }

        let mut component_slots: [Option<&mut Box<dyn Component>>; WITH] = array::from_fn(|_| None);
        for (index, slot) in component_slots.iter_mut().enumerate() {
            // ew
            unsafe {
                *slot = Some(
                    ((self.get_component(with[index])?) as *const Box<dyn Component>
                        as *mut Box<dyn Component>)
                        .as_mut()?,
                )
            }
        }

        Some(array::from_fn(|index| {
            component_slots[index].take().unwrap()
        }))
    }
}

/// A container that is nearly identical to a [`ComponentBundle`], with the one difference being that it is only able to
/// store [`SerializableComponents`](SerializableComponent). This allows it to be serializable and
/// implement [`Clone`] and [`PartialEq`].
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableComponentBundle {
    components: BTreeMap<ComponentId, Box<dyn SerializableComponent>>,
}

impl SerializableComponentBundle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_component(&self, component_id: ComponentId) -> bool {
        self.components.contains_key(&component_id)
    }

    pub fn get_component(
        &self,
        component_id: ComponentId,
    ) -> Option<&Box<dyn SerializableComponent>> {
        self.components.get(&component_id)
    }

    pub fn get_component_mut(
        &mut self,
        component_id: ComponentId,
    ) -> Option<&mut Box<dyn SerializableComponent>> {
        self.components.get_mut(&component_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (ComponentId, &Box<dyn SerializableComponent>)> {
        self.components
            .iter()
            .map(|(&id, component)| (id, component))
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (ComponentId, &mut Box<dyn SerializableComponent>)> {
        self.components
            .iter_mut()
            .map(|(&id, component)| (id, component))
    }

    pub fn set_component<T: SerializableComponent>(&mut self, component: T) -> Option<T> {
        if let Some(old_component) = self
            .components
            .insert(component.component_id(), Box::new(component))
        {
            return Some(*Box::<dyn Any + 'static>::downcast::<T>(old_component).ok()?);
        }

        None
    }

    pub fn delete_component(
        &mut self,
        component_id: ComponentId,
    ) -> Option<Box<dyn SerializableComponent>> {
        self.components.remove(&component_id)
    }

    pub fn query<const WITH: usize, const WITHOUT: usize>(
        &self,
        with: [ComponentId; WITH],
        without: [ComponentId; WITHOUT],
    ) -> Option<[&Box<dyn SerializableComponent>; WITH]> {
        if with.is_empty() {
            return None;
        }

        for &excluded_component_id in without.iter() {
            if self.has_component(excluded_component_id) {
                return None;
            }
        }

        let mut component_slots: [Option<&Box<dyn SerializableComponent>>; WITH] =
            array::from_fn(|_| None);
        for (index, slot) in component_slots.iter_mut().enumerate() {
            *slot = Some(self.get_component(with[index])?)
        }

        Some(array::from_fn(|index| component_slots[index].unwrap()))
    }

    pub fn query_mut<const WITH: usize, const WITHOUT: usize>(
        &mut self,
        with: [ComponentId; WITH],
        without: [ComponentId; WITHOUT],
    ) -> Option<[&mut Box<dyn SerializableComponent>; WITH]> {
        if with.is_empty() {
            return None;
        }

        for &excluded_component_id in without.iter() {
            if self.has_component(excluded_component_id) {
                return None;
            }
        }

        let mut component_slots: [Option<&mut Box<dyn SerializableComponent>>; WITH] =
            array::from_fn(|_| None);
        for (index, slot) in component_slots.iter_mut().enumerate() {
            // ew
            unsafe {
                *slot = Some(
                    ((self.get_component(with[index])?) as *const Box<dyn SerializableComponent>
                        as *mut Box<dyn SerializableComponent>)
                        .as_mut()?,
                )
            }
        }

        Some(array::from_fn(|index| {
            component_slots[index].take().unwrap()
        }))
    }
}

#[macro_export]
macro_rules! query_bundle {
    ($bundle:expr, ($($with:ty),*), ($($without:ty),*)) => {
        ::paste::paste! {
            $bundle.query([$(<$with>::COMPONENT_ID),*], [$(<$without>::COMPONENT_ID),*]).map(|[$([<$with:snake>]),*]| {
                unsafe { ($(([<$with:snake>] as *const ::std::boxed::Box<dyn hydrogen_ecs::component::Component> as *const ::std::boxed::Box<$with>).as_ref().unwrap().as_ref(),)*) }
            })
        }
    };
    ($bundle:expr, $($with:ty),*) => {
        hydrogen_ecs::component::query!($bundle, ($($with),*), ())
    };
}

#[macro_export]
macro_rules! query_bundle_mut {
    ($bundle:expr, ($($with:ty),*), ($($without:ty),*)) => {
        ::paste::paste! {
            $bundle.query_mut([$(<$with>::COMPONENT_ID),*], [$(<$without>::COMPONENT_ID),*]).map(|[$([<$with:snake>]),*]| {
                unsafe { ($(([<$with:snake>] as *const ::std::boxed::Box<dyn hydrogen_ecs::component::Component> as *mut ::std::boxed::Box<$with>).as_mut().unwrap().as_mut(),)*) }
            })
        }
    };
    ($bundle:expr, $($with:ty),*) => {
        hydrogen_ecs::component::query_mut!($bundle, ($($with),*), ())
    };
}
