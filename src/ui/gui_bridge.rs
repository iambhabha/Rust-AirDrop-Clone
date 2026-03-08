//! Bridge between the Dioxus GUI and the FastShare backend running in a background thread.
//! Allows the GUI to read nearby devices and trigger send-file requests.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::app::{AppState, TransferHistoryItem};

/// Channel message: send this file to this peer address.
pub type SendFileRequest = (PathBuf, SocketAddr);

/// Incoming file transfer progress for UI display.
#[derive(Debug, Clone)]
pub struct IncomingProgress {
    pub file_name: String,
    pub progress: f64,
    pub total_bytes: u64,
    pub received_bytes: u64,
    pub total_chunks: u64,
    pub received_chunks: u64,
}

/// Shared bridge set by the backend when ready, read by the GUI.
static BRIDGE: OnceLock<GuiBridge> = OnceLock::new();

/// Handle to backend state and send channel. Safe to read from the GUI thread.
pub struct GuiBridge {
    pub state: std::sync::Arc<AppState>,
    pub send_tx: tokio::sync::mpsc::Sender<SendFileRequest>,
}

/// Set the bridge (called once from the backend thread when App is ready).
pub fn set_bridge(state: std::sync::Arc<AppState>, send_tx: tokio::sync::mpsc::Sender<SendFileRequest>) {
    let _ = BRIDGE.set(GuiBridge { state, send_tx });
}

/// Get the bridge if the backend is ready. Returns None until the backend has started.
pub fn get_bridge() -> Option<&'static GuiBridge> {
    BRIDGE.get()
}

/// Get current outgoing transfer progress for UI (file name, size, bytes sent, progress).
pub fn get_transfer_progress() -> Option<crate::transfer::sender::TransferProgress> {
    BRIDGE
        .get()
        .and_then(|b| b.state.transfer_progress.lock().ok().and_then(|g| g.clone()))
}

/// Get transfer history (sent + received files).
pub fn get_transfer_history() -> Vec<TransferHistoryItem> {
    BRIDGE
        .get()
        .and_then(|b| b.state.transfer_history.lock().ok())
        .map(|g| g.clone())
        .unwrap_or_default()
}

/// Get progress of all active incoming transfers.
pub fn get_incoming_progress() -> Vec<IncomingProgress> {
    let Some(b) = BRIDGE.get() else {
        return Vec::new();
    };
    let mut list = Vec::new();
    for r in b.state.transfer_receiver.active_receptions().iter() {
        let s = r.value();
        let received = s.chunks_received.load(std::sync::atomic::Ordering::Relaxed);
        let progress = if s.plan.total_chunks > 0 {
            received as f64 / s.plan.total_chunks as f64
        } else {
            0.0
        };
        list.push(IncomingProgress {
            file_name: s.plan.file_name.clone(),
            progress,
            total_bytes: s.plan.total_size,
            received_bytes: (received * s.plan.chunk_size).min(s.plan.total_size),
            total_chunks: s.plan.total_chunks,
            received_chunks: received,
        });
    }
    list
}

/// Get download directory for received files.
pub fn get_download_path() -> String {
    BRIDGE
        .get()
        .map(|b| b.state.download_path.clone())
        .unwrap_or_default()
}

/// Open a file with the system default application.
pub fn open_file(path: &std::path::Path) {
    let _ = opener::open(path);
}

/// Respond to incoming transfer (Accept or Decline). Call from GUI when user clicks.
pub fn respond_incoming(file_id: &str, accept: bool) {
    if let Some(b) = BRIDGE.get() {
        if let Some((_, tx)) = b.state.pending_decisions.remove(file_id) {
            let _ = tx.send(accept);
        }
        if let Ok(mut guard) = b.state.pending_incoming_display.lock() {
            *guard = None;
        }
    }
}
