use hydrogen_data_structures::selection::Selection;
use hydrogen_ecs::component::{Component, ComponentId};
use serde::{Deserialize, Serialize};

use crate::{comm::NetMessage, server_client::ClientId};

#[derive(Debug, Component)]
pub struct Replicated {
    pub owner: Option<ClientId>,
    pub replicate_to: Selection<ClientId>,
    pub permitted_writes: Selection<ComponentId>,
}

#[derive(Debug, Serialize, Deserialize, NetMessage)]
pub enum EcsNetEvent {}
