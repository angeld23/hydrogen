use std::collections::BTreeMap;

use derive_more::*;
use hydrogen_core::dyn_util::AsAny;
use hydrogen_data_structures::selection::Selection;
use hydrogen_net::{
    comm::{NetMessage, TcpCommunicator},
    server_client::ClientId,
};
use serde::{Deserialize, Serialize};

use crate::{
    component::{Component, ComponentId, SerializableComponent},
    entity::EntityId,
    query, query_one,
    world::World,
};

mod hydrogen {
    pub use crate as ecs;
    pub use hydrogen_net as net;
}

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
pub struct ServerEntityId(pub EntityId);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Component, SerializableComponent)]
pub struct Replicate {
    pub server_entity_id: ServerEntityId,
    pub owner: Option<ClientId>,
    pub replicate_to: Selection<ClientId>,
    /// Note: The client can never write to a [`Replicate`] component.
    pub client_writable: Selection<ComponentId>,
    pub replicated_components: Selection<ComponentId>,
    pub auto_replicate_changes: Selection<ComponentId>,
}

#[derive(Debug, Serialize, Deserialize, NetMessage, IsVariant, Unwrap, TryUnwrap)]
pub enum NetEcsCommand {
    SetComponent(ServerEntityId, Box<dyn SerializableComponent>),
    DeleteComponent(ServerEntityId, ComponentId),
    DeleteEntity(ServerEntityId),
}

impl NetEcsCommand {
    pub fn server_entity_id(&self) -> ServerEntityId {
        match self {
            Self::SetComponent(server_entity_id, _) => *server_entity_id,
            Self::DeleteComponent(server_entity_id, _) => *server_entity_id,
            Self::DeleteEntity(server_entity_id) => *server_entity_id,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct EcsServerReplicator {
    pub client_id: ClientId,
    pub current_entities:
        BTreeMap<ServerEntityId, BTreeMap<ComponentId, Box<dyn SerializableComponent>>>,
}

impl EcsServerReplicator {
    pub fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            current_entities: Default::default(),
        }
    }

    pub fn update(&mut self, world: &mut World, comm: &mut TcpCommunicator) {
        // make sure all relevant entities are present in current_entities
        for (entity_id, (replicate,)) in query!(world, Replicate) {
            let entity_should_exist_on_client = replicate.owner == Some(self.client_id)
                || replicate.replicate_to.contains(&self.client_id);

            if entity_should_exist_on_client {
                self.current_entities.entry(entity_id.into()).or_default();
            } else if self.current_entities.remove(&entity_id.into()).is_some() {
                comm.send(NetEcsCommand::DeleteEntity(entity_id.into()));
            }
        }

        // worst part of the borrow checker existing is having to do shit like this
        let mut entities_to_delete = Vec::<ServerEntityId>::new();
        let mut components_to_delete = Vec::<(ServerEntityId, ComponentId)>::new();

        // rectify
        for (&server_entity_id, current_components) in self.current_entities.iter_mut() {
            let entity_id = server_entity_id.0;
            if let Some((replicate,)) = query_one!(world, entity_id, Replicate) {
                for (&component_id, _) in current_components.iter() {
                    if !world.has_component(entity_id, component_id) {
                        components_to_delete.push((server_entity_id, component_id));
                    }
                }

                for (component_id, serializable_component) in
                    world.get_all_serializable_components(entity_id)
                {
                    let should_exist = replicate.replicated_components.contains(&component_id);

                    if let Some(current_component) = current_components.get_mut(&component_id) {
                        if !should_exist {
                            components_to_delete.push((server_entity_id, component_id));
                        } else {
                            // we don't want to replicate the client's own changes back to it
                            let is_self_client_writable = replicate.owner == Some(self.client_id)
                                && replicate.client_writable.contains(&component_id);

                            // always auto-replicate changes made to a Replicate component
                            let should_auto_replicate_changes = current_component.component_id()
                                == replicate.component_id()
                                || (!is_self_client_writable
                                    && replicate.auto_replicate_changes.contains(&component_id));

                            if should_auto_replicate_changes
                                && current_component != serializable_component
                            {
                                current_components
                                    .insert(component_id, serializable_component.clone());
                                comm.send(NetEcsCommand::SetComponent(
                                    server_entity_id,
                                    serializable_component.clone(),
                                ));
                            }
                        }
                    } else if should_exist {
                        current_components.insert(component_id, serializable_component.clone());
                        comm.send(NetEcsCommand::SetComponent(
                            server_entity_id,
                            serializable_component.clone(),
                        ));
                    }
                }
            } else {
                entities_to_delete.push(server_entity_id);
            }
        }

        // process any requested deletions of entities and components

        for server_entity_id in entities_to_delete {
            if self.current_entities.remove(&server_entity_id).is_some() {
                comm.send(NetEcsCommand::DeleteEntity(server_entity_id));
            }
        }

        for (server_entity_id, component_id) in components_to_delete {
            if let Some(current_components) = self.current_entities.get_mut(&server_entity_id) {
                if current_components.remove(&component_id).is_some() {
                    comm.send(NetEcsCommand::DeleteComponent(
                        server_entity_id,
                        component_id,
                    ));
                }
            }
        }
    }

    pub fn replicate(
        &mut self,
        world: &mut World,
        comm: &mut TcpCommunicator,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> bool {
        if let Some(component) = world.get_component(entity_id, component_id) {
            if let Some(serializeable_component) = component
                .as_any()
                .downcast_ref::<Box<dyn SerializableComponent>>()
            {
                comm.send(NetEcsCommand::SetComponent(
                    entity_id.into(),
                    serializeable_component.clone(),
                ));
                self.current_entities
                    .entry(entity_id.into())
                    .or_default()
                    .insert(component_id, serializeable_component.clone());
                return true;
            }
        }
        false
    }
}
