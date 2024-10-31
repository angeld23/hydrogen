use hydrogen_core::dyn_util::AsAny;

use crate::{
    component::{Component, ComponentId, ComponentSet, SerializableComponent},
    entity::EntityId,
};
use std::{array, collections::BTreeMap};

#[derive(Debug, Default)]
pub struct World {
    components: BTreeMap<ComponentId, ComponentSet>,
    next_entity_id: u32,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_entity_id(&mut self) -> EntityId {
        self.next_entity_id += 1;
        (self.next_entity_id - 1).into()
    }

    pub fn get_component(
        &self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> Option<&Box<dyn Component>> {
        self.components.get(&component_id)?.get(entity_id)
    }

    pub fn has_component(&self, entity_id: EntityId, component_id: ComponentId) -> bool {
        if let Some(component_set) = self.components.get(&component_id) {
            component_set.has_entity(entity_id)
        } else {
            false
        }
    }

    pub fn has_entity(&self, entity_id: EntityId) -> bool {
        self.components
            .iter()
            .any(move |(_, component_set)| component_set.has_entity(entity_id))
    }

    pub fn get_component_mut(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> Option<&mut Box<dyn Component>> {
        self.components.get_mut(&component_id)?.get_mut(entity_id)
    }

    pub fn get_all_components(
        &self,
        entity_id: EntityId,
    ) -> impl Iterator<Item = (ComponentId, &Box<dyn Component>)> {
        self.components
            .iter()
            .filter_map(move |(&component_id, component_set)| {
                Some((component_id, component_set.get(entity_id)?))
            })
    }

    pub fn get_all_components_mut(
        &mut self,
        entity_id: EntityId,
    ) -> impl Iterator<Item = (ComponentId, &mut Box<dyn Component>)> {
        self.components
            .iter_mut()
            .filter_map(move |(&component_id, component_set)| {
                Some((component_id, component_set.get_mut(entity_id)?))
            })
    }

    pub fn get_all_serializable_components(
        &self,
        entity_id: EntityId,
    ) -> impl Iterator<Item = (ComponentId, &Box<dyn SerializableComponent>)> {
        self.get_all_components(entity_id)
            .filter_map(|(component_id, component)| {
                Some((
                    component_id,
                    component
                        .as_any()
                        .downcast_ref::<Box<dyn SerializableComponent>>()?,
                ))
            })
    }

    pub fn get_all_serializable_components_mut(
        &mut self,
        entity_id: EntityId,
    ) -> impl Iterator<Item = (ComponentId, &mut Box<dyn SerializableComponent>)> {
        self.get_all_components_mut(entity_id)
            .filter_map(|(component_id, component)| {
                Some((
                    component_id,
                    component
                        .as_any_mut()
                        .downcast_mut::<Box<dyn SerializableComponent>>()?,
                ))
            })
    }

    pub fn set_component<T: Component>(&mut self, entity_id: EntityId, component: T) -> Option<T> {
        let component_set = if let Some(set) = self.components.get_mut(&component.component_id()) {
            set
        } else {
            self.components.insert(
                component.component_id(),
                ComponentSet::new(component.component_id()),
            );
            self.components.get_mut(&component.component_id())?
        };

        if let Some(old_component) = component_set.set(entity_id, Box::new(component)) {
            return Some(*Box::<(dyn std::any::Any + 'static)>::downcast::<T>(old_component).ok()?);
        }

        None
    }

    pub fn delete_component(
        &mut self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> Option<Box<dyn Component>> {
        self.components.get_mut(&component_id)?.delete(entity_id)
    }

    pub fn delete_entity(&mut self, entity_id: EntityId) -> bool {
        let mut found = false;
        for (_, component_set) in self.components.iter_mut() {
            if component_set.delete(entity_id).is_some() {
                found = true;
            }
        }
        found
    }

    fn required_iter_upper_bound(&self, with: &[ComponentId]) -> usize {
        if with.is_empty() {
            self.components
                .values()
                .map(|component_set| component_set.entity_component_indices().len())
                .max()
                .unwrap_or(0)
        } else {
            with.iter()
                .filter_map(|component_id| {
                    Some(
                        self.components
                            .get(component_id)?
                            .entity_component_indices()
                            .len(),
                    )
                })
                .max()
                .unwrap_or(0)
        }
    }

    pub fn query_one<const WITH: usize, const WITHOUT: usize>(
        &self,
        entity_id: EntityId,
        with: [ComponentId; WITH],
        without: [ComponentId; WITHOUT],
    ) -> Option<[&Box<dyn Component>; WITH]> {
        if with.is_empty() && !self.has_entity(entity_id) {
            return None;
        }

        for &excluded_component_id in without.iter() {
            if self.has_component(entity_id, excluded_component_id) {
                return None;
            }
        }

        let mut component_slots: [Option<&Box<dyn Component>>; WITH] = array::from_fn(|_| None);
        for (index, slot) in component_slots.iter_mut().enumerate() {
            *slot = Some(self.get_component(entity_id, with[index])?)
        }

        Some(array::from_fn(|index| component_slots[index].unwrap()))
    }

    pub fn query_one_mut<const WITH: usize, const WITHOUT: usize>(
        &mut self,
        entity_id: EntityId,
        with: [ComponentId; WITH],
        without: [ComponentId; WITHOUT],
    ) -> Option<[&mut Box<dyn Component>; WITH]> {
        if with.is_empty() && !self.has_entity(entity_id) {
            return None;
        }

        for &excluded_component_id in without.iter() {
            if self.has_component(entity_id, excluded_component_id) {
                return None;
            }
        }

        let mut component_slots: [Option<&mut Box<dyn Component>>; WITH] = array::from_fn(|_| None);
        for (index, slot) in component_slots.iter_mut().enumerate() {
            // ew
            unsafe {
                *slot = Some(
                    ((self.get_component(entity_id, with[index])?) as *const Box<dyn Component>
                        as *mut Box<dyn Component>)
                        .as_mut()?,
                )
            }
        }

        Some(array::from_fn(|index| {
            component_slots[index].take().unwrap()
        }))
    }

    pub fn query<const WITH: usize, const WITHOUT: usize>(
        &self,
        with: [ComponentId; WITH],
        without: [ComponentId; WITHOUT],
    ) -> impl Iterator<Item = (EntityId, [&Box<dyn Component>; WITH])> {
        let upper_bound = self.required_iter_upper_bound(&with);
        (0..upper_bound).filter_map(move |i| {
            let entity_id = i.into();

            Some((entity_id, self.query_one(entity_id, with, without)?))
        })
    }

    pub fn query_mut<const WITH: usize, const WITHOUT: usize>(
        &mut self,
        with: [ComponentId; WITH],
        without: [ComponentId; WITHOUT],
    ) -> impl Iterator<Item = (EntityId, [&mut Box<dyn Component>; WITH])> {
        let upper_bound = self.required_iter_upper_bound(&with);

        (0..upper_bound).filter_map(move |i| {
            let entity_id = i.into();

            if with.is_empty() && !self.has_entity(entity_id) {
                return None;
            }

            for &excluded_component_id in without.iter() {
                if self.has_component(entity_id, excluded_component_id) {
                    return None;
                }
            }

            let mut component_slots: [Option<&mut Box<dyn Component>>; WITH] =
                array::from_fn(|_| None);
            for (index, slot) in component_slots.iter_mut().enumerate() {
                // ew
                // is there an easier way to force an immutable reference to be mutable?
                unsafe {
                    *slot = Some(
                        ((self.get_component(entity_id, with[index])?) as *const Box<dyn Component>
                            as *mut Box<dyn Component>)
                            .as_mut()?,
                    )
                }
            }

            Some((
                entity_id,
                array::from_fn(|index| component_slots[index].take().unwrap()),
            ))
        })
    }
}

#[macro_export]
macro_rules! query {
    ($world:expr, ($($with:ty),*), ($($without:ty),*)) => {
        ::paste::paste! {
            $world.query([$(<$with>::COMPONENT_ID),*], [$(<$without>::COMPONENT_ID),*]).map(|(entity_id, [$([<$with:snake>]),*])| {
                unsafe { (entity_id, ($(([<$with:snake>] as *const ::std::boxed::Box<dyn hydrogen_ecs::component::Component> as *const ::std::boxed::Box<$with>).as_ref().unwrap().as_ref(),)*)) }
            })
        }
    };
    ($world:expr, $($with:ty),*) => {
        hydrogen_ecs::world::query!($world, ($($with),*), ())
    };
}

#[macro_export]
macro_rules! query_mut {
    ($world:expr, ($($with:ty),*), ($($without:ty),*)) => {
        ::paste::paste! {
            $world.query_mut([$(<$with>::COMPONENT_ID),*], [$(<$without>::COMPONENT_ID),*]).map(|(entity_id, [$([<$with:snake>]),*])| {
                unsafe { (entity_id, ($(([<$with:snake>] as *const ::std::boxed::Box<dyn hydrogen_ecs::component::Component> as *mut ::std::boxed::Box<$with>).as_mut().unwrap().as_mut(),)*)) }
            })
        }
    };
    ($world:expr, $($with:ty),*) => {
        hydrogen_ecs::world::query_mut!($world, ($($with),*), ())
    };
}

#[macro_export]
macro_rules! query_one {
    ($world:expr, $entity_id:expr, ($($with:ty),*), ($($without:ty),*)) => {
        ::paste::paste! {
            $world.query_one($entity_id, [$(<$with>::COMPONENT_ID),*], [$(<$without>::COMPONENT_ID),*]).map(|[$([<$with:snake>]),*]| {
                unsafe { ($(([<$with:snake>] as *const ::std::boxed::Box<dyn hydrogen_ecs::component::Component> as *const ::std::boxed::Box<$with>).as_ref().unwrap().as_ref(),)*) }
            })
        }
    };
    ($world:expr, $entity_id:expr, $($with:ty),*) => {
        hydrogen_ecs::world::query_one!($world, $entity_id, ($($with),*), ())
    };
}

#[macro_export]
macro_rules! query_one_mut {
    ($world:expr, $entity_id:expr, ($($with:ty),*), ($($without:ty),*)) => {
        ::paste::paste! {
            $world.query_one_mut($entity_id, [$(<$with>::COMPONENT_ID),*], [$(<$without>::COMPONENT_ID),*]).map(|[$([<$with:snake>]),*]| {
                unsafe { ($(([<$with:snake>] as *const ::std::boxed::Box<dyn hydrogen_ecs::component::Component> as *mut ::std::boxed::Box<$with>).as_mut().unwrap().as_mut(),)*) }
            })
        }
    };
    ($world:expr, $entity_id:expr, $($with:ty),*) => {
        hydrogen_ecs::world::query_one_mut!($world, $entity_id, ($($with),*), ())
    };
}

pub use {query, query_mut, query_one, query_one_mut};
