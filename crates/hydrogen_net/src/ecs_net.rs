use derive_more::*;
use hydrogen_data_structures::selection::Selection;
use hydrogen_ecs::{
    component::{Component, ComponentId, SerializableComponent},
    entity::EntityId,
};
use serde::{Deserialize, Serialize};

use crate::{comm::NetMessage, server_client::ClientId};

#[derive(Debug, Serialize, Deserialize, Component, SerializableComponent)]
pub struct Replicated {
    pub owner: Option<ClientId>,
    pub replicate_to: Selection<ClientId>,
    pub client_writable: Selection<ComponentId>,
    pub auto_replicated: Selection<ComponentId>,
}

#[derive(Debug, Serialize, Deserialize, NetMessage, IsVariant, Unwrap, TryUnwrap)]
pub enum EcsCommand {
    SetComponents(EntityId, Vec<Box<dyn SerializableComponent>>),
    DeleteComponent(EntityId, ComponentId),
    DeleteEntity(EntityId),
}
