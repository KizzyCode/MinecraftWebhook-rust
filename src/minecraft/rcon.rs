//! An implementation of the RCON API

use crate::{config::Config, error, error::Error};
use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    str,
    sync::atomic::{AtomicI32, Ordering::SeqCst},
    time::Duration,
};

/// The atomic ID counter
static ID_COUNTER: AtomicI32 = AtomicI32::new(0);

/// An RCON connection
#[derive(Debug)]
pub struct RconConnection {
    /// The underlying connection
    connection: TcpStream,
}
impl RconConnection {
    /// The metadata size within an RCON message (**excluding** the length field)
    const META_SIZE: usize = 4 + 4 + 2;
    /// The timeout of RCON connections
    const TIMEOUT: Duration = Duration::from_secs(10);
    /// The maximum size of an RCON message
    const SIZE_MAX: i32 = 4110; // https://wiki.vg/Rcon#Fragmentation

    /// Creates a new RCON connection
    pub fn new(config: &Config) -> Result<Self, Error> {
        // Parse the remote address
        let Some(address) = config.rcon.address.to_socket_addrs()?.next() else {
            return Err(error!("Failed to parse RCON address"));
        };

        // Connect and configure socket
        let connection = TcpStream::connect_timeout(&address, Self::TIMEOUT)?;
        connection.set_read_timeout(Some(Self::TIMEOUT))?;
        connection.set_write_timeout(Some(Self::TIMEOUT))?;

        // Init self and authenticate if necessary
        let mut this = Self { connection };
        if let Some(password) = &config.rcon.password {
            // Perform an authentication transaction
            this.transaction(3, password)?;
        }
        Ok(this)
    }

    /// Sends an RCON command
    pub fn send(&mut self, command: &str) -> Result<String, Error> {
        self.transaction(2, command)
    }

    /// Performs a request-response transaction
    fn transaction(&mut self, type_: i32, body: &str) -> Result<String, Error> {
        // Send message
        let id = ID_COUNTER.fetch_add(1, SeqCst);
        let request = Self::serialize(id, type_, body)?;
        self.connection.write_all(&request)?;

        // Read size field
        let mut size_bytes = [0; 4];
        self.connection.read_exact(&mut size_bytes)?;
        let size @ 0..=Self::SIZE_MAX = i32::from_le_bytes(size_bytes) else {
            // Return error
            return Err(error!("Announced RCON response is too large ({})", i32::from_le_bytes(size_bytes)));
        };

        // Prepare message buffer
        #[allow(clippy::arithmetic_side_effects, reason = "SIZE_MAX is significantly smaller than usize::MAX")]
        let mut response = Vec::with_capacity(4 + size as usize);
        response.extend(size_bytes);

        // Expand the buffer with 4 trailing `0` bytes
        #[allow(clippy::arithmetic_side_effects, reason = "SIZE_MAX is significantly smaller than usize::MAX")]
        response.resize(4 + size as usize, 0);

        // Read and parse response
        #[allow(clippy::indexing_slicing, reason = "Buffer has at least a size of 4 due to the resize")]
        self.connection.read_exact(&mut response[4..])?;
        let (response_id, _, payload) = Self::deserialize(&response)?;

        // Validate response
        let true = response_id == id else {
            // Log detailed error
            return Err(error!("Invalid RCON response ID ({response_id})"));
        };
        Ok(payload)
    }

    /// Serializes a message
    fn serialize(id: i32, type_: i32, payload: &str) -> Result<Vec<u8>, Error> {
        // Encode the size
        #[allow(clippy::arithmetic_side_effects, reason = "Payload is constrained by isize::MAX")]
        let size = i32::try_from(payload.len() + Self::META_SIZE)?;

        // Serialize the message
        #[allow(clippy::arithmetic_side_effects, reason = "Payload is constrained by isize::MAX")]
        let mut message: Vec<u8> = Vec::with_capacity(4 + Self::META_SIZE + payload.len());
        message.extend(size.to_le_bytes());
        message.extend(id.to_le_bytes());
        message.extend(type_.to_le_bytes());
        message.extend(payload.as_bytes());
        message.extend(b"\0\0");
        Ok(message)
    }

    /// Deserializes a message
    fn deserialize(message: &[u8]) -> Result<(i32, i32, String), Error> {
        // Destructure the header
        let [l0, l1, l2, l3, i0, i1, i2, i3, t0, t1, t2, t3, ..] = message else {
            return Err(error!("Truncated RCON message header"));
        };

        // Destructure header
        let size = i32::from_le_bytes([*l0, *l1, *l2, *l3]);
        let id = i32::from_le_bytes([*i0, *i1, *i2, *i3]);
        let type_ = i32::from_le_bytes([*t0, *t1, *t2, *t3]);

        // Compute body length
        let size = usize::try_from(size)?;
        let Some(body_len) = size.checked_sub(Self::META_SIZE) else {
            // Log detailed error
            return Err(error!("Invalid size field in RCON message ({size})"));
        };

        // Decode body
        let mut body = String::new();
        if body_len > 0 {
            // Decode body string
            #[allow(clippy::arithmetic_side_effects, reason = "Body length is constrained by i32::MAX")]
            let Some(bytes) = message.get(12..12 + body_len) else {
                // Log detailed error
                return Err(error!("Truncated RCON message body (expected {}, got {})", 12 + body_len, message.len()))?;
            };

            // Store body
            let body_str = str::from_utf8(bytes)?;
            body = body_str.to_string();
        }
        Ok((id, type_, body))
    }
}

/// Executes an RCON command (oneshot for `RconConnection::new` + `RconConnection::send`)
pub fn exec(config: &Config, command: &str) -> Result<String, Error> {
    let mut connection = RconConnection::new(config)?;
    connection.send(command)
}
