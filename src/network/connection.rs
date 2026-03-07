//! # QUIC Connection Manager
//!
//! Implements secure, high-performance QUIC connections using the Quinn library.
//! The QUIC protocol provides:
//!
//! - **Multiplexed streams** — Multiple concurrent streams over a single connection
//! - **Built-in TLS 1.3** — Encryption without separate TLS handshake overhead
//! - **0-RTT connection establishment** — Minimal latency on reconnection
//! - **Connection migration** — Seamless handling of network changes
//!
//! The server generates self-signed certificates for peer-to-peer use,
//! with trust established through the discovery/pairing layer.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use quinn::{
    ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig, TransportConfig,
    VarInt,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::app::AppState;
use crate::network::handshake::{self, Capabilities};
use crate::transfer::receiver::TransferReceiver;

// ── Constants ──

/// Maximum concurrent bidirectional streams per connection.
/// This directly controls parallel chunk transfer capacity.
const MAX_CONCURRENT_BIDI_STREAMS: u32 = 64;

/// Maximum concurrent unidirectional streams per connection.
const MAX_CONCURRENT_UNI_STREAMS: u32 = 64;

/// Initial window size in bytes (16 MB for high-throughput transfers).
const INITIAL_WINDOW: u32 = 16 * 1024 * 1024;

/// Maximum window size in bytes (64 MB for ultra-fast networks).
const MAX_WINDOW: u32 = 64 * 1024 * 1024;

/// Maximum idle timeout in milliseconds.
const MAX_IDLE_TIMEOUT_MS: u32 = 30_000;

/// Keep-alive interval in milliseconds.
const KEEP_ALIVE_INTERVAL_MS: u64 = 5_000;

// ── QUIC Server ──

/// The QUIC server that accepts incoming connections and handles
/// file transfer streams.
#[derive(Clone)]
pub struct QuicServer {
    /// The QUIC endpoint (both server and client capable)
    endpoint: Endpoint,
    /// Server listening address
    listen_addr: SocketAddr,
}

impl QuicServer {
    /// Create a new QUIC server with self-signed TLS certificates.
    ///
    /// # Arguments
    /// * `addr` — Socket address to bind to (e.g., `0.0.0.0:5000`)
    ///
    /// # Certificate Generation
    /// Generates a self-signed certificate using rcgen. In P2P mode,
    /// trust is established through the discovery/pairing layer rather
    /// than a certificate authority.
    pub async fn new(addr: SocketAddr) -> Result<Self> {
        // ── Generate Self-Signed Certificate ──
        let cert_params = rcgen::CertificateParams::new(vec!["fastshare.local".into()])
            .context("Failed to create certificate params")?;
        let key_pair = rcgen::KeyPair::generate().context("Failed to generate key pair")?;
        let cert_key = cert_params
            .self_signed(&key_pair)
            .context("Failed to generate certificate")?;

        let cert_der = cert_key.der().clone();
        let key_der_bytes = key_pair.serialize_der();

        // ── Configure Transport ──
        let mut transport = TransportConfig::default();
        transport
            .max_concurrent_bidi_streams(VarInt::from_u32(MAX_CONCURRENT_BIDI_STREAMS))
            .max_concurrent_uni_streams(VarInt::from_u32(MAX_CONCURRENT_UNI_STREAMS))
            .initial_mtu(1200)
            .max_idle_timeout(Some(
                quinn::IdleTimeout::try_from(std::time::Duration::from_millis(
                    MAX_IDLE_TIMEOUT_MS as u64,
                ))
                .unwrap(),
            ))
            .keep_alive_interval(Some(std::time::Duration::from_millis(
                KEEP_ALIVE_INTERVAL_MS,
            )))
            .receive_window(VarInt::from_u32(MAX_WINDOW))
            .send_window(INITIAL_WINDOW as u64)
            .stream_receive_window(VarInt::from_u32(INITIAL_WINDOW));

        let transport = Arc::new(transport);

        // ── Server Config ──
        let mut server_crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![cert_der.clone()],
                PrivatePkcs8KeyDer::from(key_der_bytes).into(),
            )
            .context("Failed to create server TLS config")?;
        server_crypto.alpn_protocols = vec![b"fastshare/1".to_vec()];

        let mut server_config = ServerConfig::with_crypto(Arc::new(
            quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto)
                .context("Failed to create QUIC server config")?,
        ));
        server_config.transport_config(transport.clone());

        // ── Create Endpoint ──
        let endpoint =
            Endpoint::server(server_config, addr).context("Failed to create QUIC endpoint")?;

        info!("QUIC endpoint created on {}", addr);

        Ok(Self {
            endpoint,
            listen_addr: addr,
        })
    }

    /// Main accept loop — listens for incoming QUIC connections
    /// and spawns a handler task for each one.
    ///
    /// Each connection runs through:
    /// 1. Accept the QUIC connection
    /// 2. Perform capability handshake
    /// 3. Handle incoming streams (file chunks, control messages)
    pub async fn accept_loop(&self, state: Arc<AppState>) -> Result<()> {
        info!(
            "QUIC server accepting connections on {}...",
            self.listen_addr
        );

        while let Some(incoming) = self.endpoint.accept().await {
            let state = state.clone();

            tokio::spawn(async move {
                match incoming.await {
                    Ok(connection) => {
                        let remote = connection.remote_address();
                        info!("📥 Incoming connection from {}", remote);

                        if let Err(e) = handle_connection(connection, state).await {
                            error!("Connection handler error for {}: {}", remote, e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to accept connection: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    /// Get the QUIC endpoint for making outgoing connections.
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Connect to a remote peer at the given address.
    ///
    /// Uses a client configuration that trusts any certificate (suitable
    /// for P2P where trust is established via pairing).
    pub async fn connect_to_peer(&self, addr: SocketAddr) -> Result<Connection> {
        let client_config = configure_client()?;

        let connection = self
            .endpoint
            .connect_with(client_config, addr, "fastshare.local")
            .context("Failed to initiate connection")?
            .await
            .context("Failed to establish connection")?;

        info!("📤 Connected to peer at {}", addr);

        Ok(connection)
    }
}

/// Handle a single incoming QUIC connection.
///
/// This function:
/// 1. Performs the capability handshake on the first stream
/// 2. Accepts subsequent streams for file transfer
/// 3. Routes chunks to the transfer receiver
async fn handle_connection(connection: Connection, state: Arc<AppState>) -> Result<()> {
    let remote = connection.remote_address();
    debug!("Handling connection from {}", remote);

    // ── Capability Handshake ──
    // The first bidirectional stream is used for control messages
    let (mut send, mut recv) = connection
        .accept_bi()
        .await
        .context("Failed to accept handshake stream")?;

    let remote_capabilities = handshake::receive_handshake(&mut recv).await?;
    let our_capabilities = handshake::our_capabilities();
    handshake::send_handshake(&mut send, &our_capabilities).await?;

    info!(
        "🤝 Handshake complete with {} (protocol v{}, max_streams: {})",
        remote, remote_capabilities.protocol_version, remote_capabilities.max_streams
    );

    // ── Accept Transfer Streams ──
    // Each subsequent bidirectional stream carries a file chunk
    let receiver = TransferReceiver::new(state.chunk_storage.clone());

    loop {
        match connection.accept_bi().await {
            Ok((send_stream, recv_stream)) => {
                let receiver = receiver.clone();
                tokio::spawn(async move {
                    if let Err(e) = receiver.handle_chunk_stream(recv_stream, send_stream).await {
                        warn!("Chunk stream error: {}", e);
                    }
                });
            }
            Err(quinn::ConnectionError::ApplicationClosed(_)) => {
                info!("Connection closed by peer {}", remote);
                break;
            }
            Err(e) => {
                warn!("Connection error with {}: {}", remote, e);
                break;
            }
        }
    }

    Ok(())
}

/// Create a client configuration that accepts any server certificate.
///
/// In a P2P system, trust is established through the pairing layer
/// rather than certificate authorities. This mirrors how AirDrop works.
fn configure_client() -> Result<ClientConfig> {
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipCertVerification))
        .with_no_client_auth();

    let mut client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
            .context("Failed to create QUIC client config")?,
    ));

    // ── Transport tuning for high throughput ──
    let mut transport = TransportConfig::default();
    transport
        .max_concurrent_bidi_streams(VarInt::from_u32(MAX_CONCURRENT_BIDI_STREAMS))
        .max_concurrent_uni_streams(VarInt::from_u32(MAX_CONCURRENT_UNI_STREAMS))
        .receive_window(VarInt::from_u32(MAX_WINDOW))
        .send_window(INITIAL_WINDOW as u64)
        .stream_receive_window(VarInt::from_u32(INITIAL_WINDOW))
        .keep_alive_interval(Some(std::time::Duration::from_millis(
            KEEP_ALIVE_INTERVAL_MS,
        )));

    client_config.transport_config(Arc::new(transport));

    Ok(client_config)
}

/// Certificate verifier that accepts any certificate.
/// Trust is established at the application layer through pairing.
#[derive(Debug)]
struct SkipCertVerification;

impl rustls::client::danger::ServerCertVerifier for SkipCertVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
        ]
    }
}
