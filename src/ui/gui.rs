//! Dioxus GUI for FastShare

use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use dioxus::prelude::*;
use qrcode::QrCode;

use crate::network::discovery::DeviceInfo;
use crate::ui::gui_bridge;

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_throughput(bps: u64) -> String {
    format!("{} MB/s", (bps as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0)
}

fn format_size_pct(pct: f64) -> String {
    format!("{:.1}%", pct)
}

fn progress_pct(prog: &crate::transfer::sender::TransferProgress) -> f64 {
    if prog.total_bytes == 0 {
        100.0
    } else {
        (prog.bytes_sent as f64 / prog.total_bytes as f64) * 100.0
    }
}

/// Generate QR code PNG as base64 data URL for display.
fn qr_data_url(ip: &str) -> String {
    let code = match QrCode::new(ip.as_bytes()) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let image = code
        .render::<image::Luma<u8>>()
        .quiet_zone(true)
        .min_dimensions(200, 200)
        .build();
    let png_data = Vec::new();
    let mut cursor = std::io::Cursor::new(png_data);
    if image::DynamicImage::ImageLuma8(image)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .is_err()
    {
        return String::new();
    }
    let png_data = cursor.into_inner();
    format!("data:image/png;base64,{}", BASE64.encode(png_data))
}

#[component]
fn ReceivedFileItem(item: crate::app::TransferHistoryItem) -> Element {
    let fname = item.file_name.clone();
    let is_success = item.status == "Success";
    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; padding: 0.6rem; background: #0f3460; border-radius: 8px; gap: 0.5rem;",
            div {
                style: "flex: 1; min-width: 0;",
                p { style: "margin: 0; font-size: 0.9rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{item.file_name}" }
                p { style: "margin: 0.2rem 0 0; font-size: 0.75rem; color: #a9b5c9;",
                    "{format_size(item.size)}  •  {item.status}  •  {item.timestamp}"
                }
            }
            if is_success {
                button {
                    style: "flex-shrink: 0; padding: 0.4rem 0.8rem; border-radius: 6px; background: #4ECDC4; color: #1a1a2e; border: none; cursor: pointer; font-size: 0.85rem; font-weight: bold;",
                    onclick: move |_| {
                        let path = std::path::PathBuf::from(gui_bridge::get_download_path()).join(&fname);
                        if path.exists() {
                            gui_bridge::open_file(&path);
                        }
                    },
                    "Open"
                }
            }
        }
    }
}

#[component]
fn DeviceButton(
    device: DeviceInfo,
    selected_device: Signal<Option<Option<DeviceInfo>>>,
) -> Element {
    let dev = device.clone();
    let features_str = device.supported_features.join(", ");
    rsx! {
        button {
            style: "padding: 0.8rem 1rem; text-align: left; border-radius: 8px; background-color: #0f3460; color: white; border: 1px solid #4ECDC4; cursor: pointer; width: 100%;",
            onclick: move |_| {
                selected_device.set(Some(Some(dev.clone())));
            },
            div {
                style: "font-weight: bold; margin-bottom: 0.3rem;",
                "{device.device_name}"
                span { style: "color: #4ECDC4; font-weight: normal; font-size: 0.85rem; margin-left: 0.5rem;", "({device.device_type})" }
            }
            div { style: "font-size: 0.85rem; color: #a9b5c9;",
                "IP: {device.ip_address}:{device.port}"
            }
            div { style: "font-size: 0.75rem; color: #888; margin-top: 0.2rem; font-family: monospace;",
                "ID: {device.device_id}"
            }
            div { style: "font-size: 0.75rem; color: #888;",
                "Bandwidth: {device.max_bandwidth}  •  v{device.protocol_version}"
            }
            if !device.supported_features.is_empty() {
                div { style: "font-size: 0.7rem; color: #666; margin-top: 0.2rem;",
                    "Features: {features_str}"
                }
            }
        }
    }
}

#[component]
fn DeviceCard(device: DeviceInfo) -> Element {
    let features_str = device.supported_features.join(", ");
    rsx! {
        div {
            style: "background-color: #0f3460; padding: 1rem; border-radius: 8px; border: 1px solid #4ECDC4;",
            div {
                style: "font-weight: bold; color: white; margin-bottom: 0.3rem;",
                "📱 {device.device_name}"
                span { style: "color: #4ECDC4; font-weight: normal; font-size: 0.85rem; margin-left: 0.5rem;", "({device.device_type})" }
            }
            div { style: "font-size: 0.9rem; color: #a9b5c9;",
                "IP: {device.ip_address}:{device.port}"
            }
            div { style: "font-size: 0.8rem; color: #888; margin-top: 0.3rem; font-family: monospace;",
                "Device ID: {device.device_id}"
            }
            div { style: "font-size: 0.8rem; color: #888;",
                "Bandwidth: {device.max_bandwidth}  •  Protocol v{device.protocol_version}"
            }
            if !device.supported_features.is_empty() {
                div { style: "font-size: 0.75rem; color: #666; margin-top: 0.3rem;",
                    "Features: {features_str}"
                }
            }
        }
    }
}

/// Main Dioxus application component.
pub fn app() -> Element {
    let mut selected_files = use_signal(|| Vec::<PathBuf>::new());
    let mut selected_device = use_signal(|| None::<Option<DeviceInfo>>);
    let mut send_screen = use_signal(|| false);
    let mut status_message = use_signal(|| None::<String>);
    let refresh_tick = use_signal(|| 0u32);
    let progress_tick = use_signal(|| 0u32);

    // Poll every 1.5s so Nearby Devices list updates when phone is discovered
    use_effect(move || {
        let mut tick = refresh_tick;
        spawn(async move {
            for _ in 0..400 {
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                tick.set(tick() + 1);
            }
        });
    });
    // Poll progress every 200ms when transferring
    use_effect(move || {
        let mut tick = progress_tick;
        spawn(async move {
            for _ in 0..3000 {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                tick.set(tick() + 1);
            }
        });
    });

    let transfer_progress = gui_bridge::get_transfer_progress();
    let incoming_progress = gui_bridge::get_incoming_progress();
    let transfer_history = gui_bridge::get_transfer_history();
    let incoming_history: Vec<_> = transfer_history
        .iter()
        .filter(|h| h.is_incoming)
        .cloned()
        .collect();

    let bridge = gui_bridge::get_bridge();
    let mut devices: Vec<DeviceInfo> = if let Some(ref b) = bridge {
        match b.state.nearby_devices.try_read() {
            Ok(guard) => guard.clone(),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };
    // Sort by IP for consistent ordering
    devices.sort_by(|a, b| a.ip_address.to_string().cmp(&b.ip_address.to_string()));
    let local_ip = local_ip_address::local_ip()
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

    let pending = bridge.and_then(|b| {
        b.state.pending_incoming_display.try_lock().ok().and_then(|g| g.clone())
    });
    let (pending_file_id, pending_from, pending_fname, pending_total_files) = pending
        .as_ref()
        .map(|p| (p.0.clone(), format!("{}", p.1), p.2.clone(), p.3))
        .unwrap_or((String::new(), String::new(), String::new(), 1));
    let show_incoming = pending.is_some();
    let pending_id_accept = pending_file_id.clone();
    let pending_id_decline = pending_file_id.clone();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; background-color: #1a1a2e; color: white; font-family: system-ui, sans-serif; padding: 2rem;",
            h1 {
                style: "font-size: 3rem; margin-bottom: 0.5rem; background: linear-gradient(45deg, #FF6B6B, #4ECDC4); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;",
                "⚡ FastShare"
            }
            p {
                style: "font-size: 1.2rem; margin-bottom: 2rem; color: #a9b5c9;",
                "Ultra-High-Performance P2P File Transfer"
            }

            if send_screen() {
                // ── Send screen: files + device selection ──
                div {
                    style: "background-color: #16213e; padding: 1.5rem; border-radius: 12px; width: 100%; max-width: 500px; margin-bottom: 1.5rem;",
                    h3 { style: "margin-top: 0; margin-bottom: 1rem;", "📤 Send File(s)" }
                    if selected_files().is_empty() {
                        p { style: "color: #a9b5c9;", "No files selected" }
                    } else {
                        p {
                            style: "color: #a9b5c9; margin-bottom: 0.5rem;",
                            "Files: {selected_files().len()} selected"
                        }
                        ul {
                            style: "color: #a9b5c9; font-size: 0.9rem; margin: 0.3rem 0; padding-left: 1.2rem; max-height: 100px; overflow-y: auto;",
                            for path in selected_files().iter().take(10) {
                                li {
                                    "{path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string_lossy().into_owned())}"
                                }
                            }
                            if selected_files().len() > 10 {
                                li { "... and {selected_files().len() - 10} more" }
                            }
                        }
                    }
                    p { style: "margin-top: 0.5rem; margin-bottom: 0.5rem; color: #a9b5c9;", "Select device:" }
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem; max-height: 320px; overflow-y: auto;",
                        for dinfo in &devices {
                            DeviceButton {
                                device: dinfo.clone(),
                                selected_device: selected_device,
                            }
                        }
                        if devices.is_empty() {
                            p { style: "color: #888; font-style: italic;", "No nearby devices. Start FastShare on another device on the same network." }
                        }
                    }
                    div {
                        style: "display: flex; gap: 0.5rem; margin-top: 1rem;",
                        button {
                            style: "padding: 0.6rem 1.2rem; border-radius: 8px; background-color: #4ECDC4; color: #1a1a2e; border: none; cursor: pointer; font-weight: bold;",
                            onclick: move |_| {
                                if let (Some(Some(ref device)), Some(b)) =
                                    (selected_device(), gui_bridge::get_bridge())
                                {
                                    let paths = selected_files();
                                    if paths.is_empty() {
                                        status_message.set(Some("Select one or more files first.".into()));
                                        return;
                                    }
                                    let addr = std::net::SocketAddr::new(device.ip_address, device.port);
                                    let mut sent = 0;
                                    for path in paths.iter() {
                                        if b.send_tx.try_send((path.clone(), addr)).is_ok() {
                                            sent += 1;
                                        }
                                    }
                                    if sent > 0 {
                                        status_message.set(Some(format!("Sending {} file(s)...", sent)));
                                        send_screen.set(false);
                                        selected_files.set(Vec::new());
                                        selected_device.set(None);
                                    } else {
                                        status_message.set(Some("Send queue full, try again.".into()));
                                    }
                                } else {
                                    status_message.set(Some("Select file(s) and a device first.".into()));
                                }
                            },
                            "Send"
                        }
                        button {
                            style: "padding: 0.6rem 1.2rem; border-radius: 8px; background-color: transparent; color: #4ECDC4; border: 2px solid #4ECDC4; cursor: pointer;",
                            onclick: move |_| {
                                send_screen.set(false);
                                selected_files.set(Vec::new());
                                selected_device.set(None);
                            },
                            "Cancel"
                        }
                    }
                }
            } else {
                // ── Home: main buttons ──
                div {
                    style: "display: flex; gap: 1rem; margin-top: 1rem;",
                    button {
                        style: "padding: 0.8rem 2rem; font-size: 1rem; font-weight: bold; border-radius: 8px; background-color: #4ECDC4; color: #1a1a2e; border: none; cursor: pointer;",
                        onclick: move |_| {
                            let mut sel_files = selected_files;
                            let mut show_send = send_screen;
                            spawn(async move {
                                let files = rfd::AsyncFileDialog::new().pick_files().await;
                                if let Some(paths) = files {
                                    let list: Vec<PathBuf> = paths.into_iter().map(|f| f.path().to_path_buf()).collect();
                                    if !list.is_empty() {
                                        sel_files.set(list);
                                        show_send.set(true);
                                    }
                                }
                            });
                        },
                        "Send File(s)"
                    }
                    button {
                        style: "padding: 0.8rem 2rem; font-size: 1rem; font-weight: bold; border-radius: 8px; background-color: transparent; color: #4ECDC4; border: 2px solid #4ECDC4; cursor: pointer;",
                        "Receive File"
                    }
                }
            }

            // ── IP Address + QR Code (always visible, even before backend starts) ──
            div {
                style: "margin-top: 2rem; background-color: #0f3460; padding: 1.5rem; border-radius: 12px; width: 100%; max-width: 600px; border: 2px solid #4ECDC4;",
                h3 { style: "margin-top: 0; margin-bottom: 0.5rem; color: #4ECDC4;", "📍 Enter this IP on your phone to connect" }
                div {
                    style: "display: flex; align-items: center; gap: 1.5rem; flex-wrap: wrap;",
                    div {
                        style: "flex-shrink: 0;",
                        img {
                            style: "width: 140px; height: 140px; border-radius: 8px; background: white; padding: 8px;",
                            src: "{qr_data_url(&local_ip.to_string())}",
                            alt: "QR Code",
                        }
                    }
                    div {
                        style: "flex: 1; min-width: 150px;",
                        p {
                            style: "color: white; font-size: 1.4rem; font-weight: bold; margin: 0.5rem 0; font-family: monospace;",
                            "{local_ip}"
                        }
                        p {
                            style: "color: #a9b5c9; font-size: 0.9rem; margin-bottom: 0;",
                            "Scan QR with phone or paste IP in Target Device field"
                        }
                    }
                }
            }

            div {
                style: "margin-top: 1.5rem; background-color: #16213e; padding: 1.5rem; border-radius: 12px; width: 100%; max-width: 600px;",
                h3 { style: "margin-top: 0;", "Nearby Devices" }
                span { style: "display: none", "{refresh_tick()}" }
                if bridge.is_some() {
                    if devices.is_empty() {
                        p { style: "color: #a9b5c9; font-style: italic;", "Scanning for devices..." }
                    } else {
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.8rem;",
                            for dinfo in &devices {
                                DeviceCard { device: dinfo.clone() }
                            }
                        }
                    }
                } else {
                    p { style: "color: #a9b5c9; font-style: italic;", "Starting backend..." }
                }
            }

            if let Some(ref prog) = transfer_progress {
                div {
                    style: "margin-top: 1rem; background-color: #16213e; padding: 1rem; border-radius: 12px; width: 100%; max-width: 500px; border: 1px solid #4ECDC4;",
                    p { style: "margin: 0 0 0.5rem; font-weight: bold; color: #4ECDC4;",
                        if prog.total_files > 1 {
                            "File {prog.current_file_index} of {prog.total_files}: {prog.file_name}"
                        } else {
                            "{prog.file_name}"
                        }
                    }
                    p { style: "margin: 0 0 0.5rem; font-size: 0.9rem; color: #a9b5c9;",
                        "Size: {format_size(prog.total_bytes)}  •  Sent: {format_size(prog.bytes_sent)} / {format_size(prog.total_bytes)}"
                    }
                    div {
                        style: "width: 100%; height: 8px; background: #0f3460; border-radius: 4px; overflow: hidden;",
                        div {
                            style: "height: 100%; background: linear-gradient(90deg, #4ECDC4, #44a08d); transition: width 0.15s ease; width: {format_size_pct(progress_pct(prog))};",
                        }
                    }
                    if prog.throughput_bps > 0 {
                        p { style: "margin: 0.5rem 0 0; font-size: 0.8rem; color: #888;",
                            "Throughput: {format_throughput(prog.throughput_bps)}"
                        }
                    }
                    if prog.complete {
                        p { style: "margin: 0.5rem 0 0; color: #4ECDC4; font-weight: bold;", "✅ Complete" }
                    }
                    span { style: "display: none", "{progress_tick()}" }
                }
            }

            // ── Incoming transfer progress (files being received) ──
            if !incoming_progress.is_empty() {
                div {
                    style: "margin-top: 1rem; background-color: #16213e; padding: 1rem; border-radius: 12px; width: 100%; max-width: 500px; border: 1px solid #4ECDC4;",
                    h3 { style: "margin-top: 0; margin-bottom: 0.8rem; color: #4ECDC4;", "📥 Receiving {incoming_progress.len()} file(s)" }
                    for ip in &incoming_progress {
                        div {
                            style: "margin-bottom: 0.8rem; padding: 0.5rem; background: #0f3460; border-radius: 8px;",
                            p { style: "margin: 0 0 0.3rem; font-size: 0.9rem;", "{ip.file_name}" }
                            p { style: "margin: 0 0 0.3rem; font-size: 0.8rem; color: #a9b5c9;",
                                "{format_size(ip.received_bytes)} / {format_size(ip.total_bytes)}  •  {ip.received_chunks}/{ip.total_chunks} chunks"
                            }
                            div {
                                style: "width: 100%; height: 6px; background: #1a1a2e; border-radius: 3px; overflow: hidden;",
                                div {
                                    style: "height: 100%; background: linear-gradient(90deg, #4ECDC4, #44a08d); width: {format_size_pct(ip.progress * 100.0)};",
                                }
                            }
                        }
                    }
                    span { style: "display: none", "{progress_tick()}" }
                }
            }

            // ── Received Files (history + Open) ──
            if !incoming_history.is_empty() {
                div {
                    style: "margin-top: 1rem; background-color: #16213e; padding: 1rem; border-radius: 12px; width: 100%; max-width: 500px;",
                    h3 { style: "margin-top: 0; margin-bottom: 0.8rem; color: #4ECDC4;", "📂 Received Files ({incoming_history.len()})" }
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.5rem; max-height: 280px; overflow-y: auto;",
                        for item in incoming_history.iter().rev() {
                            ReceivedFileItem { item: item.clone() }
                        }
                    }
                    span { style: "display: none", "{refresh_tick()}" }
                }
            }
            if let Some(ref msg) = status_message() {
                p {
                    style: "margin-top: 1rem; color: #4ECDC4;",
                    "{msg}"
                }
            }

            if show_incoming {
                div {
                    style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
                    div {
                        style: "background: #16213e; padding: 2rem; border-radius: 16px; border: 2px solid #4ECDC4; max-width: 400px;",
                        h3 { style: "margin-top: 0; color: #4ECDC4;", "📥 Incoming {pending_total_files} File(s)" }
                        p { style: "color: #e0e0e0; margin: 0.5rem 0;", "From: {pending_from}" }
                        p { style: "color: #e0e0e0; margin: 0.5rem 0;", "First file: {pending_fname}" }
                        p { style: "color: #a9b5c9; font-size: 0.9rem;", "{pending_total_files} file(s) • Accept to receive" }
                        div {
                            style: "display: flex; gap: 1rem; margin-top: 1.5rem;",
                            button {
                                style: "padding: 0.6rem 1.5rem; border-radius: 8px; background: #4ECDC4; color: #1a1a2e; border: none; cursor: pointer; font-weight: bold;",
                                onclick: move |_| {
                                    gui_bridge::respond_incoming(&pending_id_accept, true);
                                },
                                "Accept"
                            }
                            button {
                                style: "padding: 0.6rem 1.5rem; border-radius: 8px; background: transparent; color: #FF6B6B; border: 2px solid #FF6B6B; cursor: pointer; font-weight: bold;",
                                onclick: move |_| {
                                    gui_bridge::respond_incoming(&pending_id_decline, false);
                                },
                                "Decline"
                            }
                        }
                    }
                }
            }
        }
    }
}
