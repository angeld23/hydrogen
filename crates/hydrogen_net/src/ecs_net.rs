use hydrogen_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::{comm::NetMessage, server_client::ClientId};

#[derive(Debug, Component)]
pub struct Networked {
    pub owner: Option<ClientId>,
}

#[derive(Debug, Serialize, Deserialize, NetMessage)]
pub enum EcsEvent {}
