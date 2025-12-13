use std::{collections::BTreeMap, sync::Mutex};

use hydrogen_core::events::EventSender;
use serde::{Deserialize, Serialize};

use crate::{
    component::{ComponentId, SerializableComponent},
    entity::EntityId,
    world::World,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentTrackerEvent<T: ?Sized = dyn SerializableComponent> {
    Added(Box<T>),
    Changed { old: Box<T>, new: Box<T> },
    Removed(Box<T>),
}

#[derive(Debug, Clone)]
pub struct EntityComponentTracker {
    pub entity_id: EntityId,
    pub component_id: ComponentId,
    pub previous_value: Option<Box<dyn SerializableComponent>>,
    pub initialized: bool,
}

impl EntityComponentTracker {
    pub fn new(entity_id: EntityId, component_id: ComponentId) -> Self {
        Self {
            entity_id,
            component_id,
            previous_value: None,
            initialized: false,
        }
    }

    pub fn update(&mut self, ecs_world: &World) -> Option<ComponentTrackerEvent> {
        let initialized = self.initialized;
        self.initialized = true;

        let current_value = ecs_world
            .get_component(self.entity_id, self.component_id)
            .and_then(|c| c.as_serializable());
        let previous_value = self.previous_value.as_deref();

        let (new_previous_value, event) = match (previous_value, current_value) {
            (None, Some(current)) => (
                Some(current.clone_box()),
                ComponentTrackerEvent::Added(current.clone_box()),
            ),
            (Some(previous), None) => (None, ComponentTrackerEvent::Removed(previous.clone_box())),
            (Some(previous), Some(current)) if previous != current => (
                Some(current.clone_box()),
                ComponentTrackerEvent::Changed {
                    old: previous.clone_box(),
                    new: current.clone_box(),
                },
            ),
            _ => return None,
        };

        self.previous_value = new_previous_value;
        initialized.then_some(event)
    }
}

type TrackerSenderPair = (
    EntityComponentTracker,
    EventSender<ComponentTrackerEvent<dyn SerializableComponent>>,
);

#[derive(Debug, Default)]
pub struct GlobalComponentTracker {
    entity_tracker_maps: Mutex<BTreeMap<EntityId, BTreeMap<ComponentId, TrackerSenderPair>>>,
}

impl GlobalComponentTracker {
    pub fn clean(&self, ecs_world: &World) {
        self.entity_tracker_maps
            .try_lock()
            .unwrap()
            .retain(|&entity_id, trackers| {
                if !ecs_world.has_entity(entity_id) {
                    return false;
                }

                trackers.retain(|_, (_, event_sender)| event_sender.receiver_count() > 0);

                if trackers.is_empty() {
                    return false;
                }

                true
            });
    }

    pub fn update(&self, ecs_world: &World) {
        self.clean(ecs_world);

        for (_, trackers) in self.entity_tracker_maps.try_lock().unwrap().iter_mut() {
            for (_, (tracker, event_sender)) in trackers.iter_mut() {
                if let Some(event) = tracker.update(ecs_world) {
                    event_sender.send(event);
                }
            }
        }
    }

    pub fn update_entity(&self, ecs_world: &World, entity_id: EntityId) {
        if let Some(trackers) = self
            .entity_tracker_maps
            .try_lock()
            .unwrap()
            .get_mut(&entity_id)
        {
            for (_, (tracker, event_sender)) in trackers.iter_mut() {
                if let Some(event) = tracker.update(ecs_world) {
                    event_sender.send(event);
                }
            }
        }
    }

    pub fn update_entity_component(
        &self,
        ecs_world: &World,
        entity_id: EntityId,
        component_id: ComponentId,
    ) {
        if let Some(trackers) = self
            .entity_tracker_maps
            .try_lock()
            .unwrap()
            .get_mut(&entity_id)
            && let Some((tracker, event_sender)) = trackers.get_mut(&component_id)
            && let Some(event) = tracker.update(ecs_world)
        {
            event_sender.send(event);
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that `T` is the concrete type associated with `component_id`.
    ///
    /// Always just use `T::COMPONENT_ID` for the `component_id` argument.
    pub unsafe fn get_event_sender_typed<T: ?Sized>(
        &'_ self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> &'_ EventSender<ComponentTrackerEvent<T>> {
        let mut maps = self.entity_tracker_maps.try_lock().unwrap();
        let pair = maps
            .entry(entity_id)
            .or_default()
            .entry(component_id)
            .or_insert_with(|| {
                (
                    EntityComponentTracker::new(entity_id, component_id),
                    EventSender::default(),
                )
            });

        unsafe {
            std::mem::transmute::<
                &EventSender<ComponentTrackerEvent<dyn SerializableComponent>>,
                &EventSender<ComponentTrackerEvent<T>>,
            >(&pair.1)
        }
    }

    pub fn get_event_sender(
        &self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> &EventSender<ComponentTrackerEvent<dyn SerializableComponent>> {
        // SAFETY:
        // Using `get_event_sender_typed::<dyn SerializableComponent>` means that get_event_sender_typed's unsafe call to `transmute` is a no-op.
        unsafe { self.get_event_sender_typed::<dyn SerializableComponent>(entity_id, component_id) }
    }
}
