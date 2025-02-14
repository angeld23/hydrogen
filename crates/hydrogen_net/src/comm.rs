use derive_more::*;
use log::error;
use serde::{Deserialize, Serialize};
use std::{
    any::Any,
    collections::VecDeque,
    io::{self, Read, Write},
    net::{Shutdown, TcpStream},
};
use thiserror::Error;

pub use hydrogen_net_proc_macro::NetMessage;

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
pub struct NetMessageId(pub u64);

#[typetag::serde(tag = "type")]
pub trait NetMessage: Any + Send + Sync {
    fn net_id(&self) -> NetMessageId;
    fn display_name(&self) -> &'static str;
}

#[derive(Debug, Error)]
pub enum TcpCommunicatorError {
    #[error("reader closed")]
    ReaderClosed,
    #[error("read IO error: {0}")]
    ReadIoError(io::Error),
    #[error("read buffer overflow, message exceeds max size of {0}")]
    ReadBufferOverflow(usize),
    #[error("reader failed to deserialize incoming message: {0}")]
    ReadDeserializeError(postcard::Error),
    #[error("writer failed to serialize outgoing message: {0}")]
    WriteSerializeError(postcard::Error),
    #[error("write IO error: {0}")]
    WriteIoError(io::Error),
}

pub struct TcpCommunicator {
    pub stream: TcpStream,
    pub max_message_size: usize,
    read_queue: VecDeque<Box<dyn NetMessage>>,
    write_queue: VecDeque<Box<dyn NetMessage>>,
    read_buffer: Vec<u8>,
    read_position: usize,
    write_buffer: Vec<u8>,
    pre_write_buffer: Vec<u8>,
    closed: bool,
}

impl std::fmt::Debug for TcpCommunicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TcpCommunicator")
            .field("stream", &self.stream)
            .field("max_message_size", &self.max_message_size)
            .field("read_queue", &format!("({} msgs)", self.read_queue.len()))
            .field("write_queue", &format!("({} msgs)", self.write_queue.len()))
            .field("read_buffer", &self.read_buffer)
            .field("read_position", &self.read_position)
            .field("write_buffer", &self.write_buffer)
            .field("pre_write_buffer", &self.pre_write_buffer)
            .field("closed", &self.closed)
            .finish()
    }
}

impl TcpCommunicator {
    pub fn new(stream: TcpStream, max_message_size: usize) -> Self {
        stream.set_nonblocking(true).unwrap();

        Self {
            stream,
            max_message_size,
            read_queue: VecDeque::default(),
            write_queue: VecDeque::default(),
            read_buffer: vec![0; max_message_size], // never resize this vec
            read_position: 0,
            write_buffer: Vec::with_capacity(max_message_size),
            pre_write_buffer: vec![0; max_message_size],
            closed: false,
        }
    }

    pub fn send(&mut self, data: impl NetMessage) {
        self.write_queue.push_back(Box::new(data));
    }

    pub fn send_boxed(&mut self, data: Box<dyn NetMessage>) {
        self.write_queue.push_back(data);
    }

    pub fn recv(&mut self) -> Option<Box<dyn NetMessage>> {
        self.read_queue.pop_front()
    }

    pub fn recv_all(&mut self) -> Vec<Box<dyn NetMessage>> {
        self.read_queue.drain(..).collect()
    }

    pub fn close(&mut self) -> bool {
        if self.closed {
            return false;
        }

        self.closed = true;
        let _ = self.stream.shutdown(Shutdown::Both);

        true
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn update(&mut self) -> Result<(), TcpCommunicatorError> {
        // read any new bytes
        'read_loop: loop {
            let bytes_read = match self
                .stream
                .read(&mut self.read_buffer[self.read_position..])
            {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        return Err(TcpCommunicatorError::ReaderClosed);
                    }
                    bytes_read
                }
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        return Err(TcpCommunicatorError::ReadIoError(e));
                    }
                    0
                }
            };

            let old_read_position = self.read_position;
            self.read_position += bytes_read;

            let stream_cleared = self.read_position < self.max_message_size;

            let mut message_start = 0usize;
            for index in old_read_position..self.read_position {
                let byte = self.read_buffer[index];
                if byte == 0 {
                    match postcard::from_bytes_cobs(&mut self.read_buffer[message_start..index + 1])
                    {
                        Ok(message) => {
                            self.read_queue.push_back(message);
                        }
                        Err(e) => return Err(TcpCommunicatorError::ReadDeserializeError(e)),
                    }
                    message_start = index + 1;
                } else if message_start == 0 && index >= self.max_message_size - 1 {
                    self.read_position = 0;
                    return Err(TcpCommunicatorError::ReadBufferOverflow(
                        self.max_message_size,
                    ));
                }
            }

            if message_start < self.max_message_size {
                self.read_buffer
                    .copy_within(message_start..self.read_position, 0);
            }
            self.read_position -= message_start;

            if stream_cleared {
                break 'read_loop;
            }
        }

        // write all queued requests
        if !self.write_queue.is_empty() {
            for message in self.write_queue.drain(..) {
                match postcard::to_slice_cobs(&message, &mut self.pre_write_buffer) {
                    Ok(slice) => self.write_buffer.extend_from_slice(slice),
                    Err(e) => return Err(TcpCommunicatorError::WriteSerializeError(e)),
                }
            }
        }

        // send bytes
        if !self.write_buffer.is_empty() {
            match self.stream.write(&self.write_buffer) {
                Ok(bytes_written) => {
                    self.write_buffer.drain(..bytes_written);
                }
                Err(e) => {
                    if e.kind() != io::ErrorKind::WouldBlock {
                        return Err(TcpCommunicatorError::WriteIoError(e));
                    }
                }
            }
        }

        Ok(())
    }
}
