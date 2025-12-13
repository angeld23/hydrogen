use std::{any::Any, array, collections::BTreeMap};

use hydrogen_core::events::EventSender;
use hydrogen_net::server_client::ClientId;

use crate::{
    change_tracker::{ComponentTrackerEvent, GlobalComponentTracker},
    component::{Component, ComponentId, ComponentSet, SerializableComponent},
    ecs_net::{NetEcsCommand, Replicate, ServerEntityId},
    entity::EntityId,
};

mod hydrogen {
    pub use crate as ecs;
}

#[derive(Debug, Default)]
pub struct World {
    components: BTreeMap<ComponentId, ComponentSet>,
    server_entity_id_map: BTreeMap<ServerEntityId, EntityId>,
    next_entity_id: u32,
    change_tracker: GlobalComponentTracker,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_entity_id(&mut self) -> EntityId {
        self.next_entity_id += 1;
        (self.next_entity_id - 1).into()
    }

    pub fn entity_id_from_server(&mut self, server_entity_id: ServerEntityId) -> EntityId {
        if let Some(&entity_id) = self.server_entity_id_map.get(&server_entity_id) {
            entity_id
        } else {
            let entity_id = self.new_entity_id();
            self.server_entity_id_map
                .insert(server_entity_id, entity_id);
            entity_id
        }
    }

    pub fn execute_net_command(&mut self, command: NetEcsCommand) {
        let entity_id = self.entity_id_from_server(command.server_entity_id());
        match command {
            NetEcsCommand::SetComponent(_, component) => {
                self.set_component_boxed(entity_id, component);
            }
            NetEcsCommand::DeleteComponent(_, component_id) => {
                self.delete_component(entity_id, component_id);
            }
            NetEcsCommand::DeleteEntity(_) => {
                self.delete_entity(entity_id);
            }
        }
    }

    pub fn execute_client_net_command(&mut self, client_id: ClientId, command: NetEcsCommand) {
        // clients cannot remove components or entities
        if let NetEcsCommand::SetComponent(server_entity_id, component) = command {
            let entity_id = server_entity_id.0;

            // Replicate components are protected
            if component.component_id() == Replicate::COMPONENT_ID {
                return;
            }

            if let Some((replicate,)) = query_one!(self, entity_id, Replicate) {
                // they need to own the entity and the component needs to be in the list of writable components

                if replicate.owner != Some(client_id) {
                    return;
                }
                if !replicate
                    .client_writable
                    .contains(&component.component_id())
                {
                    return;
                }

                self.set_component_boxed(entity_id, component);
            }
        }
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
    ) -> impl Iterator<Item = (ComponentId, &dyn SerializableComponent)> {
        self.get_all_components(entity_id)
            .filter_map(|(component_id, component)| {
                Some((component_id, component.as_serializable()?))
            })
    }

    pub fn get_all_serializable_components_mut(
        &mut self,
        entity_id: EntityId,
    ) -> impl Iterator<Item = (ComponentId, &mut dyn SerializableComponent)> {
        self.get_all_components_mut(entity_id)
            .filter_map(|(component_id, component)| {
                Some((component_id, component.as_serializable_mut()?))
            })
    }

    pub fn set_component<T: Component>(&mut self, entity_id: EntityId, component: T) -> Option<T> {
        if let Some(old_component) = self.set_component_boxed(entity_id, Box::new(component)) {
            return Some(*Box::<dyn Any + 'static>::downcast::<T>(old_component).ok()?);
        }

        None
    }

    pub fn set_component_boxed(
        &mut self,
        entity_id: EntityId,
        component: Box<dyn Component>,
    ) -> Option<Box<dyn Component>> {
        let component_set = if let Some(set) = self.components.get_mut(&component.component_id()) {
            set
        } else {
            self.components.insert(
                component.component_id(),
                ComponentSet::new(component.component_id()),
            );
            self.components.get_mut(&component.component_id())?
        };

        component_set.set(entity_id, component)
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

    pub fn update_change_tracker(&self) {
        self.change_tracker.update(self);
    }

    pub fn update_entity_change_tracker(&self, entity_id: EntityId) {
        self.change_tracker.update_entity(self, entity_id);
    }

    pub fn update_entity_component_change_tracker(
        &self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) {
        self.change_tracker
            .update_entity_component(self, entity_id, component_id);
    }

    /// # Safety
    ///
    /// The caller must ensure that `T` is the concrete type associated with `component_id`.
    ///
    /// Always just use `T::COMPONENT_ID` for the `component_id` argument.
    pub unsafe fn get_component_changed_event_sender_typed<T>(
        &self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> &EventSender<ComponentTrackerEvent<T>> {
        unsafe {
            self.change_tracker
                .get_event_sender_typed::<T>(entity_id, component_id)
        }
    }

    pub fn get_component_changed_event_sender(
        &self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> &EventSender<ComponentTrackerEvent<dyn SerializableComponent>> {
        self.change_tracker
            .get_event_sender(entity_id, component_id)
    }

    pub fn get_entity_from_component(&self, component: &impl Component) -> Option<EntityId> {
        self.components
            .get(&component.component_id())?
            .get_entity_from_component(component)
    }
}

#[macro_export]
macro_rules! query {
    ($world:expr, ($($with:ty),*), ($($without:ty),*)) => {
        ::paste::paste! {
            $world.query([$(<$with>::COMPONENT_ID),*], [$(<$without>::COMPONENT_ID),*]).map(|(entity_id, [$([<$with:snake>]),*])| {
                unsafe { (entity_id, ($(([<$with:snake>] as *const ::std::boxed::Box<dyn hydrogen::ecs::component::Component> as *const ::std::boxed::Box<$with>).as_ref().unwrap().as_ref(),)*)) }
            })
        }
    };
    ($world:expr, $($with:ty),*) => {
        hydrogen::ecs::world::query!($world, ($($with),*), ())
    };
}

#[macro_export]
macro_rules! query_mut {
    ($world:expr, ($($with:ty),*), ($($without:ty),*)) => {
        ::paste::paste! {
            $world.query_mut([$(<$with>::COMPONENT_ID),*], [$(<$without>::COMPONENT_ID),*]).map(|(entity_id, [$([<$with:snake>]),*])| {
                unsafe { (entity_id, ($(([<$with:snake>] as *const ::std::boxed::Box<dyn hydrogen::ecs::component::Component> as *mut ::std::boxed::Box<$with>).as_mut().unwrap().as_mut(),)*)) }
            })
        }
    };
    ($world:expr, $($with:ty),*) => {
        hydrogen::ecs::world::query_mut!($world, ($($with),*), ())
    };
}

#[macro_export]
macro_rules! query_one {
    ($world:expr, $entity_id:expr, ($($with:ty),*), ($($without:ty),*)) => {
        ::paste::paste! {
            $world.query_one($entity_id, [$(<$with>::COMPONENT_ID),*], [$(<$without>::COMPONENT_ID),*]).map(|[$([<$with:snake>]),*]| {
                unsafe { ($(([<$with:snake>] as *const ::std::boxed::Box<dyn hydrogen::ecs::component::Component> as *const ::std::boxed::Box<$with>).as_ref().unwrap().as_ref(),)*) }
            })
        }
    };
    ($world:expr, $entity_id:expr, $($with:ty),*) => {
        hydrogen::ecs::world::query_one!($world, $entity_id, ($($with),*), ())
    };
}

#[macro_export]
macro_rules! query_one_mut {
    ($world:expr, $entity_id:expr, ($($with:ty),*), ($($without:ty),*)) => {
        ::paste::paste! {
            $world.query_one_mut($entity_id, [$(<$with>::COMPONENT_ID),*], [$(<$without>::COMPONENT_ID),*]).map(|[$([<$with:snake>]),*]| {
                unsafe { ($(([<$with:snake>] as *const ::std::boxed::Box<dyn hydrogen::ecs::component::Component> as *mut ::std::boxed::Box<$with>).as_mut().unwrap().as_mut(),)*) }
            })
        }
    };
    ($world:expr, $entity_id:expr, $($with:ty),*) => {
        hydrogen::ecs::world::query_one_mut!($world, $entity_id, ($($with),*), ())
    };
}

#[macro_export]
macro_rules! get_component_changed_event_sender {
    ($world:expr, $entity_id:expr, $component:ty) => {
        unsafe {
            $world.get_component_changed_event_sender_typed::<$component>(
                $entity_id,
                <$component>::COMPONENT_ID,
            )
        }
    };
}

pub use {get_component_changed_event_sender, query, query_mut, query_one, query_one_mut};
