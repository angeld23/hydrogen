use std::collections::BTreeMap;

use derive_more::*;
use hydrogen_data_structures::selection::Selection;
use hydrogen_net::{comm::NetMessage, server_client::ClientId};
use serde::{Deserialize, Serialize};

use crate::{
    self as hydrogen_ecs,
    component::{Component, ComponentId, SerializableComponent},
    entity::EntityId,
};

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
pub struct Replicated {
    pub server_entity_id: ServerEntityId,
    pub owner: Option<ClientId>,
    pub replicate_to: Selection<ClientId>,
    pub client_writable: Selection<ComponentId>,
    pub auto_replicate: Selection<ComponentId>,
}

#[derive(Debug, Serialize, Deserialize, NetMessage, IsVariant, Unwrap, TryUnwrap)]
pub enum NetEcsCommand {
    SetComponent(ServerEntityId, Box<dyn SerializableComponent>),
    DeleteComponent(ServerEntityId, ComponentId),
    DeleteEntity(ServerEntityId),
}

#[derive(Debug, PartialEq)]
pub struct EcsClientReplicator {
    client_id: ClientId,
    current_components:
        BTreeMap<ServerEntityId, BTreeMap<ComponentId, Box<dyn SerializableComponent>>>,
}
