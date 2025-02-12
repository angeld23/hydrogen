use derive_more::*;
use hydrogen_core::events::EventSender;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io,
    net::{SocketAddr, TcpListener},
    rc::Rc,
    sync::{Mutex, MutexGuard},
};

use crate::comm::{TcpCommunicator, TcpCommunicatorError};

#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, From, Into, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct ClientId(pub u32);

impl Default for ClientId {
    fn default() -> Self {
        Self::generate()
    }
}

impl ClientId {
    pub fn generate() -> Self {
        Self(rand::random())
    }
}

#[derive(Debug)]
pub struct ConnectedClient {
    client_id: ClientId,
    socket_address: SocketAddr,
    pub comm: Rc<Mutex<TcpCommunicator>>,
}

impl ConnectedClient {
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub fn socket_address(&self) -> SocketAddr {
        self.socket_address
    }

    pub fn comm(&self) -> MutexGuard<'_, TcpCommunicator> {
        self.comm.lock().unwrap()
    }
}

#[derive(Debug, Unwrap, TryUnwrap, IsVariant)]
pub enum ServerEvent {
    ClientAdded(ClientId),
    ClientRemoved(ConnectedClient),
    ClientCommUpdateError(ClientId, TcpCommunicatorError),
}

#[derive(Debug)]
pub struct Server {
    pub connected_clients: BTreeMap<ClientId, ConnectedClient>,
    pub tcp_listener: TcpListener,
    pub max_message_size: usize,
    pub events: EventSender<ServerEvent>,
}

impl Server {
    pub fn new(address: SocketAddr, max_message_size: usize) -> io::Result<Self> {
        let tcp_listener = TcpListener::bind(address)?;
        tcp_listener.set_nonblocking(true)?;

        Ok(Self {
            connected_clients: BTreeMap::new(),
            tcp_listener,
            max_message_size,
            events: Default::default(),
        })
    }

    pub fn accept_connections(&mut self) -> Result<(), io::Error> {
        'accept_connection_loop: loop {
            match self.tcp_listener.accept() {
                Ok((stream, address)) => {
                    debug!("new connection from {}", address);

                    let comm = TcpCommunicator::new(stream, self.max_message_size);
                    let client_id = ClientId::generate();
                    self.connected_clients.insert(
                        client_id,
                        ConnectedClient {
                            client_id,
                            socket_address: address,
                            comm: Rc::new(Mutex::new(comm)),
                        },
                    );

                    self.events.send(ServerEvent::ClientAdded(client_id));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break 'accept_connection_loop;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    pub fn remove_client(&mut self, client_id: ClientId) -> bool {
        if let Some(client) = self.connected_clients.remove(&client_id) {
            client.comm().close();
            self.events.send(ServerEvent::ClientRemoved(client));

            return true;
        }

        false
    }

    pub fn update(&mut self) -> io::Result<()> {
        self.accept_connections()?;

        let mut clients_to_remove = Vec::<ClientId>::new();
        for client in self.connected_clients.values_mut() {
            if client.comm().is_closed() {
                clients_to_remove.push(client.client_id);
            } else if let Err(e) = client.comm().update() {
                self.events
                    .send(ServerEvent::ClientCommUpdateError(client.client_id, e));
            }
        }

        for client_id in clients_to_remove {
            self.remove_client(client_id);
        }

        Ok(())
    }
}
