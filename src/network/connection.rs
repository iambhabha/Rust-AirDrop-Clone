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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use quinn::{
    ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig, TransportConfig,
    VarInt,
};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use tracing::{debug, error, info, warn};

use tokio::sync::oneshot;

use crate::app::AppState;
use crate::network::handshake;
// use crate::transfer::receiver::TransferReceiver;

// ── Constants ──

/// Maximum concurrent bidirectional streams per connection.
const MAX_CONCURRENT_BIDI_STREAMS: u32 = 64;

/// Maximum concurrent unidirectional streams per connection.
const MAX_CONCURRENT_UNI_STREAMS: u32 = 64;

/// Initial window size in bytes (16 MB).
const INITIAL_WINDOW: u32 = 16 * 1024 * 1024;

/// Maximum window size in bytes (64 MB).
const MAX_WINDOW: u32 = 64 * 1024 * 1024;

/// Maximum idle timeout in milliseconds.
const MAX_IDLE_TIMEOUT_MS: u32 = 30_000;

/// Keep-alive interval in milliseconds.
const KEEP_ALIVE_INTERVAL_MS: u64 = 5_000;

// ── QUIC Server ──

#[derive(Clone)]
pub struct QuicServer {
    endpoint: Endpoint,
    listen_addr: SocketAddr,
}

impl QuicServer {
    pub async fn new(addr: SocketAddr) -> Result<Self> {
        let cert_params = rcgen::CertificateParams::new(vec!["fastshare.local".into()])
            .context("Failed to create certificate params")?;
        let key_pair = rcgen::KeyPair::generate().context("Failed to generate key pair")?;
        let cert_key = cert_params
            .self_signed(&key_pair)
            .context("Failed to generate certificate")?;

        let cert_der = cert_key.der().clone();
        let key_der_bytes = key_pair.serialize_der();

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

        let endpoint =
            Endpoint::server(server_config, addr).context("Failed to create QUIC endpoint")?;

        info!("QUIC endpoint created on {}", addr);

        Ok(Self {
            endpoint,
            listen_addr: addr,
        })
    }

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

    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

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

    pub async fn connect_and_handshake(
        &self,
        addr: SocketAddr,
        state: Arc<AppState>,
    ) -> Result<Connection> {
        let connection = self.connect_to_peer(addr).await?;

        let (mut send, mut recv) = connection
            .open_bi()
            .await
            .context("Failed to open handshake stream")?;

        let mut ours = handshake::our_capabilities();
        ours.device_id = state.device_id.clone();

        handshake::send_handshake(&mut send, &ours).await?;
        let theirs = handshake::receive_handshake(&mut recv).await?;

        info!(
            "🤝 Handshake complete with {} (protocol v{}, max_streams: {})",
            addr, theirs.protocol_version, theirs.max_streams
        );

        let _ = send.finish();

        Ok(connection)
    }
}

async fn handle_connection(connection: Connection, state: Arc<AppState>) -> Result<()> {
    let remote = connection.remote_address();
    debug!("Handling connection from {}", remote);

    // ── Capability Handshake ──
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
    let receiver = state.transfer_receiver.clone();

    // session_decision is a shared state for this connection:
    // None = no decision yet, Some(true) = accepted, Some(false) = declined
    let session_decision = Arc::new(tokio::sync::RwLock::new(None));
    let decision_notify = Arc::new(tokio::sync::Notify::new());

    loop {
        match connection.accept_bi().await {
            Ok((send_stream, mut recv_stream)) => {
                let receiver = receiver.clone();
                let app_state = state.clone();
                let session_decision = session_decision.clone();
                let decision_notify = decision_notify.clone();

                tokio::spawn(async move {
                    let mut type_buf = [0u8; 1];
                    if let Err(_) = recv_stream.read_exact(&mut type_buf).await {
                        return;
                    }

                    if type_buf[0] == 0x01 {
                        // File Plan
                        let mut len_buf = [0u8; 4];
                        if recv_stream.read_exact(&mut len_buf).await.is_err() {
                            return;
                        }
                        let len = u32::from_be_bytes(len_buf) as usize;
                        let mut json_buf = vec![0u8; len];
                        if recv_stream.read_exact(&mut json_buf).await.is_err() {
                            return;
                        }
                        let plan: crate::transfer::chunker::FileChunkPlan =
                            match serde_json::from_slice(&json_buf) {
                                Ok(p) => p,
                                Err(_) => return,
                            };

                        let file_id = plan.file_id.clone();
                        let file_name = plan.file_name.clone();

                        info!(
                            "📥 [FastShare] Metadata received for '{}' (ID: {}, Chunks: {})",
                            file_name, file_id, plan.total_chunks
                        );

                        if let Err(e) = receiver.handle_file_plan(plan.clone()).await {
                            error!(
                                "❌ [FastShare] Failed to handle file plan for {}: {}",
                                file_name, e
                            );
                            return;
                        }

                        // Check if session is already decided or being decided.
                        // Only the FIRST file in the batch sets pending_incoming_display and waits for user.
                        // Other files in the same batch just wait for the decision.
                        // Check if session is already decided or being decided.
                        // Only the FIRST file in the batch sets pending_incoming_display and waits for user.
                        // Other files in the same batch just wait for the decision.
                        loop {
                            let decision_val = session_decision.read().await;
                            if decision_val.is_some() {
                                break;
                            }
                            drop(decision_val);

                            let already_pending = {
                                if let Ok(guard) = app_state.pending_incoming_display.lock() {
                                    guard.is_some()
                                } else {
                                    false
                                }
                            };

                            if already_pending {
                                // Someone else is showing a popup. Wait and try again.
                                // We use a small sleep to avoid tight-looping, but naturally
                                // it will break once our session is decided by a previous stream
                                // OR when the other connection's popup clears.
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                continue;
                            }

                            let mut decision_guard = session_decision.write().await;
                            if decision_guard.is_some() {
                                break;
                            } // Session decided while we waited

                            // We grabbed the write lock and the global slot is free. Show our popup.
                            let (tx, rx) = oneshot::channel();
                            app_state.pending_decisions.insert(file_id.clone(), tx);
                            if let Ok(mut guard) = app_state.pending_incoming_display.lock() {
                                *guard = Some((
                                    file_id.clone(),
                                    remote,
                                    file_name.clone(),
                                    plan.total_files,
                                ));
                                info!(
                                    "📥 [FastShare] Set pending_incoming_display for {}",
                                    file_name
                                );
                            }

                            info!(
                                "📥 [FastShare] Incoming batch from {}: {} ({} files) — waiting for Accept/Decline",
                                remote, file_name, plan.total_files
                            );

                            drop(decision_guard); // Allow other streams in our connection to see we are working

                            let decision =
                                match tokio::time::timeout(std::time::Duration::from_secs(120), rx)
                                    .await
                                {
                                    Ok(Ok(true)) => true,
                                    _ => false,
                                };

                            // Store decision and notify others in our connection
                            let mut decision_guard = session_decision.write().await;
                            *decision_guard = Some(decision);
                            decision_notify.notify_waiters();

                            app_state.pending_decisions.remove(&file_id);
                            if let Ok(mut guard) = app_state.pending_incoming_display.lock() {
                                *guard = None;
                                info!("📥 [FastShare] Cleared pending_incoming_display for {} (Decision: {})", file_name, decision);
                            }
                            break;
                        }

                        // Just in case some stream arrived late after the decision was made but before notify_waiters
                        let final_accepted = session_decision.read().await.unwrap_or(false);

                        if !final_accepted {
                            info!("Transfer declined for {}", file_name);
                            // Add to history as declined
                            let mut history = app_state.transfer_history.lock().unwrap();
                            history.push(crate::app::TransferHistoryItem {
                                file_name: file_name.clone(),
                                size: plan.total_size,
                                status: "Declined".into(),
                                timestamp: chrono::Local::now()
                                    .format("%Y-%m-%d %H:%M:%S")
                                    .to_string(),
                                is_incoming: true,
                            });
                            return;
                        }

                        let rx = receiver.clone();
                        let app_state_inner = app_state.clone();

                        tokio::spawn(async move {
                            if let Some(rx_state) = rx.get_reception(&file_id) {
                                rx_state.completion_notify.notified().await;
                                let out_dir =
                                    std::path::PathBuf::from(&app_state_inner.download_path);
                                std::fs::create_dir_all(&out_dir).unwrap_or(());
                                let out_path = out_dir.join(&file_name);
                                if let Err(e) = rx.reassemble_file(&file_id, &out_path).await {
                                    error!("Reassembly failed for {}: {}", file_id, e);
                                    // Add to history as failed
                                    let mut history =
                                        app_state_inner.transfer_history.lock().unwrap();
                                    history.push(crate::app::TransferHistoryItem {
                                        file_name: file_name.clone(),
                                        size: plan.total_size,
                                        status: format!("Failed: {}", e),
                                        timestamp: chrono::Local::now()
                                            .format("%Y-%m-%d %H:%M:%S")
                                            .to_string(),
                                        is_incoming: true,
                                    });
                                } else {
                                    info!("File saved to {}", out_path.display());
                                    // Add to history as success
                                    let mut history =
                                        app_state_inner.transfer_history.lock().unwrap();
                                    history.push(crate::app::TransferHistoryItem {
                                        file_name: file_name.clone(),
                                        size: plan.total_size,
                                        status: "Success".into(),
                                        timestamp: chrono::Local::now()
                                            .format("%Y-%m-%d %H:%M:%S")
                                            .to_string(),
                                        is_incoming: true,
                                    });
                                }
                            }
                        });
                    } else if type_buf[0] == 0x02 {
                        // Chunk Data
                        if let Err(e) = receiver.handle_chunk_stream(recv_stream, send_stream).await
                        {
                            warn!("Chunk stream error: {}", e);
                        }
                    } else {
                        warn!("Unknown stream type: {}", type_buf[0]);
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

fn configure_client() -> Result<ClientConfig> {
    let mut crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(SkipCertVerification))
        .with_no_client_auth();
    crypto.alpn_protocols = vec![b"fastshare/1".to_vec()];

    let mut client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
            .context("Failed to create QUIC client config")?,
    ));

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
