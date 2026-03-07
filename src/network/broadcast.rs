//! # UDP Broadcast Engine
//!
//! Provides high-performance UDP broadcast capabilities for fast LAN-wide
//! device discovery. Uses platform-specific socket options for optimal
//! broadcast performance.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use tokio::net::UdpSocket;
use tracing::debug;

/// UDP broadcast engine for LAN-wide discovery announcements.
///
/// Creates a broadcast-capable UDP socket and sends packets to the
/// LAN broadcast address (255.255.255.255) on the configured port.
pub struct BroadcastEngine {
    /// The port to broadcast on
    port: u16,
}

impl BroadcastEngine {
    /// Create a new broadcast engine on the specified port.
    pub fn new(port: u16) -> Result<Self> {
        Ok(Self { port })
    }

    /// Broadcast data to all devices on the LAN.
    ///
    /// Sends the packet to 255.255.255.255 on the configured port,
    /// which will be received by all devices listening on that port.
    pub async fn broadcast(&self, data: &[u8]) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;

        let broadcast_addr: SocketAddr = format!("255.255.255.255:{}", self.port).parse()?;

        socket.send_to(data, broadcast_addr).await?;

        debug!("Broadcast sent: {} bytes to {}", data.len(), broadcast_addr);
        Ok(())
    }

    /// Send a directed unicast packet to a specific device.
    ///
    /// Used for direct device-to-device communication after discovery,
    /// such as responding to a discovery query.
    pub async fn send_to(&self, data: &[u8], target: SocketAddr) -> Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.send_to(data, target).await?;

        debug!("Unicast sent: {} bytes to {}", data.len(), target);
        Ok(())
    }

    /// Listen for broadcast packets on the configured port.
    ///
    /// Returns `(data, source_address)` for each received packet.
    /// This is a convenience wrapper; the main discovery listener
    /// typically uses its own socket.
    pub async fn listen(&self) -> Result<(Vec<u8>, SocketAddr)> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", self.port)).await?;
        socket.set_broadcast(true)?;

        let mut buf = vec![0u8; 65535];
        let (len, src) = socket.recv_from(&mut buf).await?;

        buf.truncate(len);
        Ok((buf, src))
    }
}
