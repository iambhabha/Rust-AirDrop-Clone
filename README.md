# Rust AirDrop Clone - Complete Project Guide

This file gives a full overview of the app: what it does, which technologies are used, how code is organized, how to run it, and what work remains.

## 1) Project Summary

Rust AirDrop Clone (branding in app as **Rust Drop / FastShare**) is a peer-to-peer file transfer system.

Core idea:
- Discover nearby devices on local network.
- Establish secure QUIC connection.
- Transfer one or multiple files with progress tracking.
- Support accept/decline, pause/resume, cancel, and transfer history.
- Use Rust backend engine for performance and Flutter UI for cross-platform app UX.

## 2) Main Technology Stack

### Backend (Rust)
- Language: Rust (edition 2021)
- Async runtime: tokio
- Transport protocol: quinn (QUIC)
- TLS/security: rustls + rcgen
- Discovery: mdns-sd + UDP/broadcast utilities
- Serialization: serde + serde_json
- Compression: lz4_flex, zstd
- Integrity/checks: sha2, crc32fast
- Logging/observability: tracing, tracing-subscriber
- Error handling: anyhow, thiserror
- Other utilities: uuid, bytes, dashmap, tempfile, dirs

### Frontend (Flutter)
- Flutter + Dart
- State management: mobx + flutter_mobx
- Native Rust bridge: flutter_rust_bridge
- Device and file integrations:
  - file_picker
  - path_provider
  - permission_handler
  - mobile_scanner (QR)
  - open_filex
  - shared_preferences

### Rust <-> Flutter Integration
- Bridge tool: flutter_rust_bridge
- Bridge config file: fastshare_ui/flutter_rust_bridge.yaml
- Rust bridge crate: fastshare_ui/rust
- Generated Dart bridge output: fastshare_ui/lib/src/rust

## 3) High-Level Architecture

The project has two runnable fronts over a shared Rust engine:

1. Rust Desktop app path
- Entry: src/main.rs
- Uses Dioxus desktop UI + Rust backend subsystems.

2. Flutter app path
- Entry: fastshare_ui/lib/main.dart
- Calls Rust backend functions through flutter_rust_bridge.

Shared backend logic lives in root Rust crate modules:
- src/network: discovery, handshake, connection
- src/transfer: sender, receiver, progress, chunk handling
- src/storage: chunk/file persistence
- src/optimizer: network monitoring/adaptive controls
- src/distributed: peer/chunk distribution
- src/security: auth and safety features
- src/qr: QR generation and scanning support

## 4) Important App Features (Implemented)

- Nearby device discovery on LAN
- Secure connection and file transfer over QUIC
- Multi-file batch send
- Incoming transfer approval (accept/decline)
- Pause/resume and cancel transfer
- Outgoing + incoming progress tracking
- Transfer history persistence (history.json in download folder)
- Toggle settings:
  - checksum verification
  - compression
- Open received file/folder from UI

## 5) Folder Guide

- Cargo.toml: main Rust project dependencies and build profile
- src/: core Rust backend + desktop main
- fastshare_ui/: Flutter app
- fastshare_ui/rust/: Rust bridge library used by Flutter
- fastshare_ui/lib/stores/fastshare_store.dart: UI state + polling + actions
- fastshare_ui/lib/src/rust/api/simple.dart: generated Dart wrappers for Rust APIs

## 6) Runtime Data/Behavior

- Default backend listen address: 0.0.0.0:5000
- Device discovery is periodic and can also be manually triggered
- Flutter store uses timers:
  - discovery refresh every 10 seconds
  - progress polling every 100 ms
- Transfer history is saved/loaded from download directory under app-managed path

## 7) Prerequisites

Install these before running:

- Rust toolchain (stable)
- Flutter SDK (matching Dart >= 3.10)
- Android Studio/SDK for Android build
- (Optional desktop) Visual Studio Build Tools for Windows desktop targets

Verify:

```powershell
rustc --version
cargo --version
flutter --version
flutter doctor
```

## 8) How To Run (Recommended Flutter App)

From project root:

```powershell
cd fastshare_ui
flutter pub get
flutter run
```

Notes:
- On Android, grant storage/network/camera permissions as needed.
- Rust backend is started by Flutter app through FRB (RustLib.init + startFastshare call flow).

## 9) How To Run (Rust Desktop App)

From project root:

```powershell
cargo run --release
```

This runs the Rust desktop build (Dioxus UI + backend services).

## 10) When You Change Rust APIs (Bridge Regeneration)

If you edit exported Rust API functions in fastshare_ui/rust/src/api, regenerate FRB bindings:

```powershell
cd fastshare_ui
flutter_rust_bridge_codegen generate --config flutter_rust_bridge.yaml
```

Then run again:

```powershell
flutter pub get
flutter run
```

## 11) Build Outputs

### Flutter Android (APK)

```powershell
cd fastshare_ui
flutter build apk --release
```

### Flutter Windows desktop

```powershell
cd fastshare_ui
flutter config --enable-windows-desktop
flutter build windows --release
```

### Rust optimized binary

```powershell
cargo build --release
```

## 12) Current Engineering Notes

- Multiple historical build/analyze logs are present in fastshare_ui and fastshare_ui/rust for debugging.
- There is generated code in fastshare_ui/lib/src/rust and fastshare_ui/lib/stores/*.g.dart; avoid manual edits in generated files.
- Build profile is tuned for performance in release (LTO, strip, panic abort).

## 13) Suggested Next Work (Roadmap)

- Add auto-retry and stronger reconnect strategy for unstable Wi-Fi.
- Add richer conflict handling for duplicate filenames.
- Add resumable transfer persistence across app restarts.
- Add end-to-end integration tests for send/receive scenarios.
- Improve telemetry dashboards (latency, throughput, failure reasons).
- Add encryption key/pairing UX hardening.

## 14) Quick New Contributor Checklist

1. Read this file fully.
2. Run Flutter app first and validate device discovery.
3. Test send/receive between two devices on same network.
4. Check history, pause/resume, cancel flows.
5. Only then start modifying transfer/network modules.

---

If needed, we can also create:
- ARCHITECTURE.md (deeper technical design)
- API_REFERENCE.md (all Rust bridge functions)
- CONTRIBUTING.md (coding standards + workflow)
