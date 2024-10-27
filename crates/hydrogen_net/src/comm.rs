use log::error;
use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    net::TcpStream,
};
use thiserror::Error;

pub use hydrogen_net_proc_macro::NetMessage;

#[typetag::serde(tag = "type")]
pub trait NetMessage: 'static {}

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

pub struct TcpCommunicator<const MAX_INCOMING_MESSAGE_SIZE: usize = 32768> {
    pub stream: TcpStream,
    pub read_queue: VecDeque<Box<dyn NetMessage>>,
    pub write_queue: VecDeque<Box<dyn NetMessage>>,
    pub read_buffer: [u8; MAX_INCOMING_MESSAGE_SIZE],
    pub read_position: usize,
    pub write_buffer: Vec<u8>,
}

impl<const MAX_INCOMING_MESSAGE_SIZE: usize> TcpCommunicator<MAX_INCOMING_MESSAGE_SIZE> {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(true).unwrap();

        Self {
            stream,
            read_queue: VecDeque::default(),
            write_queue: VecDeque::default(),
            read_buffer: [0; MAX_INCOMING_MESSAGE_SIZE],
            read_position: 0,
            write_buffer: Vec::with_capacity(MAX_INCOMING_MESSAGE_SIZE),
        }
    }

    pub fn send(&mut self, data: impl NetMessage) {
        self.write_queue.push_back(Box::new(data));
    }

    pub fn recv(&mut self) -> Option<Box<dyn NetMessage>> {
        self.read_queue.pop_front()
    }

    pub fn recv_all(&mut self) -> Vec<Box<dyn NetMessage>> {
        self.read_queue.drain(..).collect()
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

            let stream_cleared = self.read_position < MAX_INCOMING_MESSAGE_SIZE;

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
                } else if message_start == 0 && index >= MAX_INCOMING_MESSAGE_SIZE - 1 {
                    self.read_position = 0;
                    return Err(TcpCommunicatorError::ReadBufferOverflow(
                        MAX_INCOMING_MESSAGE_SIZE,
                    ));
                }
            }

            if message_start < MAX_INCOMING_MESSAGE_SIZE {
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
            let mut holder = [0u8; MAX_INCOMING_MESSAGE_SIZE];
            for message in self.write_queue.drain(..) {
                match postcard::to_slice_cobs(&message, &mut holder) {
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
