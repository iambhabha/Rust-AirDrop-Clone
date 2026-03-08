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
    format!(
        "{} MB/s",
        (bps as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0
    )
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
fn HistoryItemWidget(item: crate::app::TransferHistoryItem) -> Element {
    let is_success = item.status == "Success" || item.status.to_lowercase().contains("success");
    let is_incoming = item.is_incoming;
    let status_color = if is_success { "#4ECDC4" } else { "#FF6B6B" };

    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: space-between; padding: 0.75rem; background: #0f3460; border-radius: 10px; gap: 0.8rem; border-left: 4px solid {status_color};",
            div {
                style: "display: flex; align-items: center; justify-content: center; width: 36px; height: 36px; background: rgba(255,255,255,0.05); border-radius: 50%; color: {status_color}; font-size: 1.2rem;",
                if is_incoming { "📥" } else { "📤" }
            }
            div {
                style: "flex: 1; min-width: 0;",
                p { style: "margin: 0; font-size: 0.95rem; font-weight: bold; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{item.file_name}" }
                p { style: "margin: 0.2rem 0 0; font-size: 0.8rem; color: #a9b5c9;",
                    "{format_size(item.size)}  •  {item.status}"
                }
                p { style: "margin: 0.1rem 0 0; font-size: 0.7rem; color: #666;", "{item.timestamp}" }
            }
            div {
                style: "display: flex; gap: 0.5rem;",
                if is_success && item.saved_path.is_some() {
                    button {
                        style: "padding: 0.4rem 0.7rem; border-radius: 6px; background: rgba(78, 205, 196, 0.2); color: #4ECDC4; border: 1px solid #4ECDC4; cursor: pointer; font-size: 0.8rem; font-weight: bold;",
                        onclick: move |_| {
                            if let Some(ref path_str) = item.saved_path {
                                gui_bridge::open_file(&std::path::PathBuf::from(path_str));
                            }
                        },
                        "📂 Open"
                    }
                }
            }
        }
    }
}

#[component]
fn DeviceButton(device: DeviceInfo, selected_device: Signal<Option<DeviceInfo>>) -> Element {
    let dev = device.clone();
    let is_selected = selected_device()
        .as_ref()
        .map_or(false, |d| d.device_id == device.device_id);
    let border_color = if is_selected {
        "#4ECDC4"
    } else {
        "transparent"
    };
    let bg_color = if is_selected { "#1e3a5f" } else { "#0f3460" };
    let features_str = device.supported_features.join(", ");

    rsx! {
        button {
            style: "padding: 0.8rem 1rem; text-align: left; border-radius: 8px; background-color: {bg_color}; color: white; border: 2px solid {border_color}; cursor: pointer; width: 100%; transition: all 0.2s;",
            onclick: move |_| {
                tracing::info!("Selected device: {}", dev.ip_address);
                selected_device.set(Some(dev.clone()));
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
fn DeviceCard(
    device: DeviceInfo,
    selected_device: Signal<Option<DeviceInfo>>,
    selected_files: Signal<Vec<PathBuf>>,
    send_screen: Signal<bool>,
) -> Element {
    let dev = device.clone();
    rsx! {
        div {
            style: "background-color: #0f3460; padding: 1.2rem; border-radius: 16px; border: 1px solid rgba(78, 205, 196, 0.3); display: flex; justify-content: space-between; align-items: center; box-shadow: 0 8px 24px rgba(0,0,0,0.2); transition: transform 0.2s; cursor: default;",
            div {
                div {
                    style: "font-weight: 800; color: white; margin-bottom: 0.4rem; font-size: 1.15rem;",
                    "📱 {device.device_name}"
                    span { style: "color: #4ECDC4; font-weight: normal; font-size: 0.85rem; margin-left: 0.6rem; background: rgba(78, 205, 196, 0.1); padding: 2px 8px; border-radius: 12px;", "{device.device_type}" }
                }
                div { style: "font-size: 0.95rem; color: #a9b5c9; opacity: 0.8;",
                    "IP: {device.ip_address}:{device.port}"
                }
            }
            button {
                style: "padding: 0.7rem 1.4rem; font-size: 0.95rem; font-weight: 900; border-radius: 12px; background-color: #4ECDC4; color: #1a1a2e; border: none; cursor: pointer; white-space: nowrap; transition: all 0.2s; box-shadow: 0 4px 12px rgba(78, 205, 196, 0.3);",
                onclick: move |_| {
                    let mut sel_files = selected_files;
                    let mut show_send = send_screen;
                    let mut sel_dev = selected_device;
                    let device_clone = dev.clone();
                    spawn(async move {
                        let files = rfd::AsyncFileDialog::new().pick_files().await;
                        if let Some(paths) = files {
                            let list: std::vec::Vec<std::path::PathBuf> = paths.into_iter().map(|f| f.path().to_path_buf()).collect();
                            if !list.is_empty() {
                                sel_files.set(list);
                                sel_dev.set(Some(device_clone));
                                show_send.set(true);
                            }
                        }
                    });
                },
                "📤 SEND"
            }
        }
    }
}

/// Main Dioxus application component.
pub fn app() -> Element {
    let mut selected_files = use_signal(|| Vec::<PathBuf>::new());
    let mut selected_device = use_signal(|| None::<DeviceInfo>);
    let mut send_screen = use_signal(|| false);
    let mut status_message = use_signal(|| None::<String>);
    let mut history_tab = use_signal(|| "All");
    let refresh_tick = use_signal(|| 0u32);
    let progress_tick = use_signal(|| 0u32);

    // Poll every 1.5s so Nearby Devices list updates when phone is discovered
    use_effect(move || {
        let mut tick = refresh_tick;
        spawn(async move {
            for _ in 0..1000 {
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                tick.set(tick() + 1);
            }
        });
    });
    // Poll progress every 200ms when transferring
    use_effect(move || {
        let mut tick = progress_tick;
        let mut stat = status_message;
        spawn(async move {
            for _ in 0..10000 {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                tick.set(tick() + 1);
                if let Some(msg) = gui_bridge::take_backend_status() {
                    stat.set(Some(msg));
                }
            }
        });
    });

    let transfer_progress = gui_bridge::get_transfer_progress();
    let incoming_progress = gui_bridge::get_incoming_progress();
    let transfer_history = gui_bridge::get_transfer_history();

    let filtered_history: Vec<_> = transfer_history
        .iter()
        .filter(|h| match history_tab() {
            "Received" => h.is_incoming,
            "Sent" => !h.is_incoming,
            _ => true,
        })
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
        b.state
            .pending_incoming_display
            .try_lock()
            .ok()
            .and_then(|g| g.clone())
    });
    let (pending_file_id, pending_from, pending_fname, pending_total_files, pending_total_size) =
        pending
            .as_ref()
            .map(|p| (p.0.clone(), format!("{}", p.1), p.2.clone(), p.3, p.4))
            .unwrap_or((String::new(), String::new(), String::new(), 1, 0));
    let show_incoming = pending.is_some();
    let pending_id_accept = pending_file_id.clone();
    let pending_id_decline = pending_file_id.clone();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; min-height: 100vh; background-color: #1a1a2e; color: white; font-family: 'Inter', system-ui, sans-serif; padding: 2rem; overflow-x: hidden;",

            // Header
            h1 {
                style: "font-size: 3.5rem; margin-bottom: 0.5rem; background: linear-gradient(45deg, #FF6B6B, #4ECDC4); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; font-weight: 900;",
                "⚡ FastShare"
            }
            p {
                style: "font-size: 1.25rem; margin-bottom: 2.5rem; color: #a9b5c9; letter-spacing: 1px;",
                "The Ultimate P2P File Transfer Protocol"
            }

            if send_screen() {
                // ── Send screen ──
                div {
                    style: "background-color: #16213e; padding: 2rem; border-radius: 20px; width: 100%; max-width: 500px; margin-bottom: 2rem; border: 1px solid rgba(78, 205, 196, 0.3); box-shadow: 0 20px 40px rgba(0,0,0,0.4);",
                    h3 { style: "margin-top: 0; margin-bottom: 1.5rem; color: #4ECDC4; font-size: 1.5rem;", "📤 Send Files" }
                    if selected_files().is_empty() {
                        p { style: "color: #a9b5c9; font-style: italic;", "No files selected." }
                    } else {
                        div {
                            style: "background: rgba(0,0,0,0.2); border-radius: 12px; padding: 1rem; margin-bottom: 1.5rem;",
                            p { style: "color: white; font-weight: bold; margin: 0 0 0.5rem;", "{selected_files().len()} Files Selected" }
                            ul {
                                style: "color: #a9b5c9; font-size: 0.9rem; margin: 0; padding-left: 1.2rem; max-height: 120px; overflow-y: auto;",
                                for path in selected_files().iter().take(15) {
                                    li { style: "margin-bottom: 0.2rem;",
                                        "{path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string_lossy().into_owned())}"
                                    }
                                }
                                if selected_files().len() > 15 {
                                    li { "... and {selected_files().len() - 15} more" }
                                }
                            }
                        }
                    }

                    p { style: "margin-bottom: 0.8rem; color: white; font-weight: bold;", "Select Destination Device:" }
                    div {
                        style: "display: flex; flex-direction: column; gap: 0.8rem; max-height: 250px; overflow-y: auto; padding-right: 0.5rem;",
                        for dinfo in &devices {
                            DeviceButton {
                                device: dinfo.clone(),
                                selected_device: selected_device,
                            }
                        }
                        if devices.is_empty() {
                            p { style: "color: #ff6b6b; font-size: 0.9rem; margin: 1rem 0; border: 1px dashed #ff6b6b; padding: 0.8rem; border-radius: 8px;", "No nearby devices found. Start FastShare on your phone/tablet." }
                        }
                    }
                    div {
                        style: "display: flex; gap: 1.2rem; margin-top: 2rem;",
                        button {
                            style: "flex: 2; padding: 1rem; border-radius: 12px; background-color: #4ECDC4; color: #1a1a2e; border: none; cursor: pointer; font-weight: bold; font-size: 1.1rem; transition: all 0.2s;",
                            onclick: move |_| {
                                if let (Some(ref device), Some(b)) =
                                    (selected_device(), gui_bridge::get_bridge())
                                {
                                    let paths = selected_files();
                                    if paths.is_empty() {
                                        status_message.set(Some("Please select files first.".into()));
                                        return;
                                    }
                                    let addr = std::net::SocketAddr::new(device.ip_address, device.port);
                                    let mut sent = 0;
                                    for path in paths.iter() {
                                        match b.send_tx.try_send((path.clone(), addr)) {
                                            Ok(_) => sent += 1,
                                            Err(e) => tracing::error!("Failed to queue file: {}", e),
                                        }
                                    }
                                    if sent > 0 {
                                        status_message.set(Some(format!("Sent request for {} file(s)", sent)));
                                        send_screen.set(false);
                                        selected_files.set(Vec::new());
                                        selected_device.set(None);
                                    }
                                } else {
                                    status_message.set(Some("Please select a device.".into()));
                                }
                            },
                            "🚀 Send Now"
                        }
                        button {
                            style: "flex: 1; padding: 1rem; border-radius: 12px; background-color: transparent; color: #FF6B6B; border: 2px solid #FF6B6B; cursor: pointer; font-weight: bold;",
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
                // ── Home ──
                div {
                    style: "display: flex; gap: 2rem; margin-top: 0;",
                    button {
                        style: "padding: 1.25rem 3.5rem; font-size: 1.4rem; font-weight: 900; border-radius: 20px; background-color: #4ECDC4; color: #1a1a2e; border: none; cursor: pointer; box-shadow: 0 10px 40px rgba(78, 205, 196, 0.4); transition: all 0.2s;",
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
                        "🚀 SEND FILES"
                    }
                }
            }

            // ── IP & QR ──
            div {
                style: "margin-top: 3rem; background: rgba(15, 52, 96, 0.6); backdrop-filter: blur(12px); padding: 2.2rem; border-radius: 28px; width: 100%; max-width: 650px; border: 1px solid rgba(78, 205, 196, 0.25); box-shadow: 0 12px 48px rgba(0,0,0,0.4);",
                div {
                    style: "display: flex; align-items: center; gap: 3rem; flex-wrap: wrap; justify-content: center;",
                    div {
                        style: "background: white; padding: 14px; border-radius: 20px; box-shadow: 0 0 35px rgba(255,255,255,0.15);",
                        img {
                            style: "width: 170px; height: 170px; display: block;",
                            src: "{qr_data_url(&local_ip.to_string())}",
                            alt: "QR Code",
                        }
                    }
                    div {
                        style: "flex: 1; min-width: 280px; text-align: left;",
                        h3 { style: "color: #4ECDC4; margin: 0 0 0.6rem; font-size: 1.5rem; font-weight: 800;", "Receive Files" }
                        p { style: "color: #a9b5c9; font-size: 1rem; margin: 0; font-weight: 500;", "Your Device IP Address:" }
                        p {
                            style: "color: white; font-size: 2.2rem; font-weight: 900; margin: 0.5rem 0; font-family: 'JetBrains Mono', monospace; letter-spacing: 1px; text-shadow: 0 0 20px rgba(78, 205, 196, 0.5);",
                            "{local_ip}"
                        }
                        p {
                            style: "color: #888; font-size: 0.95rem; margin-top: 1rem; line-height: 1.6; font-style: italic;",
                            "Scan QR with FastShare Mobile or enter IP to connect instantly."
                        }
                    }
                }
            }

            // ── Nearby Devices (Always Visible on Home) ──
            if !send_screen() {
                div {
                    style: "margin-top: 3.5rem; width: 100%; max-width: 650px;",
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem;",
                        h3 { style: "color: #4ECDC4; margin: 0; font-size: 1.6rem; font-weight: 800; display: flex; align-items: center; gap: 0.8rem;",
                            "📱 Nearby Devices"
                            if !devices.is_empty() {
                                span { style: "font-size: 0.8rem; background: #4ECDC4; color: #1a1a2e; padding: 2px 10px; border-radius: 12px; font-weight: 900;", "{devices.len()} ONLINE" }
                            }
                        }
                        button {
                            style: "background: rgba(78, 205, 196, 0.1); border: 1px solid rgba(78, 205, 196, 0.3); color: #4ECDC4; padding: 0.4rem 1rem; border-radius: 10px; cursor: pointer; font-size: 0.85rem; font-weight: bold; transition: all 0.2s;",
                            onclick: move |_| {
                                gui_bridge::trigger_scan();
                                status_message.set(Some("Scanning for devices...".into()));
                            },
                            "🔄 Rescan"
                        }
                    }

                    if devices.is_empty() {
                        div {
                            style: "background: rgba(15, 52, 96, 0.3); padding: 3rem; border-radius: 20px; text-align: center; border: 1px dashed rgba(78, 205, 196, 0.3);",
                            div { style: "font-size: 2.5rem; margin-bottom: 1rem; animation: pulse 2s infinite;", "🔍" }
                            p { style: "color: #a9b5c9; font-size: 1.1rem; margin: 0; font-weight: 500;", "Searching for nearby devices..." }
                            p { style: "color: #666; font-size: 0.9rem; margin-top: 0.5rem;", "Make sure FastShare is open on other devices." }
                        }
                    } else {
                        div {
                            style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1.5rem;",
                            for dinfo in &devices {
                                DeviceCard {
                                    device: dinfo.clone(),
                                    selected_device: selected_device,
                                    selected_files: selected_files,
                                    send_screen: send_screen,
                                }
                            }
                        }
                    }
                }
            }

            // ── Ongoing transfers ──
            div {
                style: "margin-top: 2.5rem; width: 100%; max-width: 650px; display: flex; flex-direction: column; gap: 1.2rem;",
                if let Some(ref prog) = transfer_progress {
                    div {
                        style: "background: #16213e; padding: 1.5rem; border-radius: 18px; border: 1px solid #4ECDC4; box-shadow: 0 10px 25px rgba(78, 205, 196, 0.1);",
                        div {
                            style: "display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 1rem;",
                            div {
                                h4 { style: "margin: 0; color: #4ECDC4; font-size: 1.1rem;", "📤 Uploading..." }
                                p { style: "margin: 0.3rem 0 0; font-weight: bold; color: white; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 400px;", "{prog.file_name}" }
                            }
                            if prog.total_files > 1 {
                                span { style: "background: #0f3460; padding: 0.3rem 0.8rem; border-radius: 20px; font-size: 0.8rem; color: #a9b5c9;", "File {prog.current_file_index} / {prog.total_files}" }
                            }
                        }
                        div {
                            style: "width: 100%; height: 12px; background: #0f3460; border-radius: 6px; overflow: hidden; margin-top: 1rem; border: 1px solid rgba(255,255,255,0.05);",
                            div {
                                style: "height: 100%; background: linear-gradient(90deg, #4ECDC4, #FF6B6B); transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1); width: {format_size_pct(progress_pct(prog))};",
                            }
                        }
                        div {
                            style: "display: flex; justify-content: space-between; margin-top: 0.8rem; font-size: 0.9rem; color: #a9b5c9; font-weight: 500;",
                            span { "{format_size_pct(progress_pct(prog))} completed" }
                            span { style: "color: #4ECDC4;", "{format_throughput(prog.throughput_bps)}" }
                        }
                    }
                }

                if !incoming_progress.is_empty() {
                    div {
                        style: "background: #16213e; padding: 1.5rem; border-radius: 18px; border: 1px solid #FF6B6B; box-shadow: 0 10px 25px rgba(255,107,107,0.1);",
                        h4 { style: "margin: 0 0 1.2rem; color: #FF6B6B; font-size: 1.1rem; display: flex; align-items: center; gap: 0.6rem;", "📥 Receiving {incoming_progress.len()} Files" }
                        div {
                            style: "display: flex; flex-direction: column; gap: 1rem;",
                            for ip in &incoming_progress {
                                div {
                                    style: "padding: 1rem; background: rgba(15, 52, 96, 0.4); border-radius: 12px; border: 1px solid rgba(255,255,255,0.05);",
                                    p { style: "margin: 0 0 0.5rem; font-size: 0.95rem; font-weight: bold; color: white;", "{ip.file_name}" }
                                    div {
                                        style: "width: 100%; height: 8px; background: #1a1a2e; border-radius: 4px; overflow: hidden; margin: 0.6rem 0;",
                                        div {
                                            style: "height: 100%; background: #FF6B6B; width: {format_size_pct(ip.progress * 100.0)};",
                                        }
                                    }
                                    div { style: "display: flex; justify-content: space-between; font-size: 0.8rem; color: #a9b5c9;",
                                        span { "{format_size_pct(ip.progress * 100.0)} • {ip.received_chunks}/{ip.total_chunks} Chunks" }
                                        span { "{format_size(ip.received_bytes)} / {format_size(ip.total_bytes)}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Transfer History ──
            div {
                style: "margin-top: 3rem; background: rgba(15, 52, 96, 0.4); backdrop-filter: blur(10px); padding: 2.2rem; border-radius: 28px; width: 100%; max-width: 650px; box-shadow: 0 18px 56px rgba(0,0,0,0.4); border: 1px solid rgba(255,255,255,0.05);",
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem;",
                    h3 { style: "margin: 0; font-size: 1.7rem; font-weight: 900;", "📋 Transfer History" }
                    div {
                        style: "display: flex; background: #0f3460; border-radius: 14px; padding: 0.4rem; box-shadow: inset 0 2px 8px rgba(0,0,0,0.3);",
                        for tab in ["All", "Received", "Sent"] {
                            button {
                                key: "{tab}",
                                style: if history_tab() == tab {
                                    "padding: 0.6rem 1.4rem; border-radius: 11px; border: none; font-size: 0.9rem; font-weight: 800; cursor: pointer; transition: all 0.25s; background: #4ECDC4; color: #1a1a2e; box-shadow: 0 4px 12px rgba(78, 205, 196, 0.3);"
                                } else {
                                    "padding: 0.6rem 1.4rem; border-radius: 11px; border: none; font-size: 0.9rem; font-weight: 700; cursor: pointer; transition: all 0.25s; background: transparent; color: #a9b5c9;"
                                },
                                onclick: move |_| history_tab.set(tab),
                                "{tab}"
                            }
                        }
                    }
                }

                div {
                    style: "display: flex; flex-direction: column; gap: 1.1rem; max-height: 450px; overflow-y: auto; padding-right: 0.8rem;",
                    if filtered_history.is_empty() {
                        div {
                            style: "text-align: center; padding: 3.5rem 1rem; color: #555; background: rgba(0,0,0,0.15); border-radius: 20px; border: 1px dashed rgba(255,255,255,0.05);",
                            p { style: "font-size: 1.2rem; margin: 0; font-style: italic;", "No transfers to show here yet." }
                        }
                    } else {
                        for item in filtered_history.iter().rev() {
                            HistoryItemWidget { item: item.clone() }
                        }
                    }
                }
            }

            // Status Snackbar
            if let Some(ref msg) = status_message() {
                div {
                    style: "position: fixed; bottom: 3rem; background: #4ECDC4; color: #1a1a2e; padding: 1.2rem 2.5rem; border-radius: 16px; font-weight: 900; font-size: 1.1rem; box-shadow: 0 15px 40px rgba(78, 205, 196, 0.4); border-top: 4px solid white; z-index: 2000; animation: snackbarIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);",
                    "✨ {msg}"
                }
            }

            // Incoming Popup
            if show_incoming {
                div {
                    style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.85); display: flex; align-items: center; justify-content: center; z-index: 3000; backdrop-filter: blur(10px); animation: fadeIn 0.3s ease-out;",
                    div {
                        style: "background: #16213e; padding: 3rem; border-radius: 32px; border: 2px solid #4ECDC4; max-width: 450px; width: 90%; box-shadow: 0 0 80px rgba(78, 205, 196, 0.25); text-align: center; overflow: hidden; position: relative;",
                        div {
                            style: "width: 100px; height: 100px; background: rgba(78, 205, 196, 0.15); border-radius: 50%; display: flex; align-items: center; justify-content: center; margin: 0 auto 2rem; font-size: 3.5rem; animation: pulse 2s infinite;",
                            "📥"
                        }
                        h2 { style: "margin-top: 0; color: #4ECDC4; font-size: 2.2rem; font-weight: 900;",
                            if pending_total_files > 1 { "Receive {pending_total_files} Files?" } else { "Receive File?" }
                        }
                        div {
                            style: "margin: 2rem 0; padding: 1.5rem; background: rgba(0,0,0,0.3); border-radius: 20px; text-align: left; border: 1px solid rgba(255,255,255,0.05);",
                            p { style: "color: #a9b5c9; margin: 0; font-size: 0.9rem; font-weight: 600; text-transform: uppercase; letter-spacing: 1px;", "From Device" }
                            p { style: "color: white; margin: 0.4rem 0 1.5rem; font-weight: 800; font-size: 1.2rem;", "{pending_from}" }
                            p { style: "color: #a9b5c9; margin: 0; font-size: 0.9rem; font-weight: 600; text-transform: uppercase; letter-spacing: 1px;", "First File" }
                            p { style: "color: white; margin: 0.4rem 0; font-weight: 700; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 1.1rem;", "{pending_fname}" }
                            if pending_total_files > 1 {
                                div { style: "margin-top: 1rem; background: #4ECDC4; color: #1a1a2e; padding: 0.5rem 1rem; border-radius: 12px; font-weight: 900; font-size: 0.9rem; display: flex; justify-content: space-between; align-items: center;",
                                    span { "+ {pending_total_files - 1} MORE" }
                                    span { style: "opacity: 0.8; font-size: 0.8rem;", "{format_size(pending_total_size)}" }
                                }
                            } else {
                                div { style: "margin-top: 0.5rem; text-align: right;",
                                    span { style: "color: #a9b5c9; font-size: 0.85rem; font-weight: 600;", "{format_size(pending_total_size)}" }
                                }
                            }
                        }
                        div {
                            style: "display: flex; gap: 1.5rem; margin-top: 2.5rem;",
                            button {
                                style: "flex: 1.5; padding: 1.2rem; border-radius: 16px; background: #4ECDC4; color: #1a1a2e; border: none; cursor: pointer; font-weight: 900; font-size: 1.2rem; box-shadow: 0 8px 20px rgba(78, 205, 196, 0.4); transition: transform 0.2s;",
                                onclick: move |_| {
                                    gui_bridge::respond_incoming(&pending_id_accept, true);
                                },
                                "ACCEPT"
                            }
                            button {
                                style: "flex: 1; padding: 1.2rem; border-radius: 16px; background: transparent; color: #FF6B6B; border: 2px solid #FF6B6B; cursor: pointer; font-weight: 800; font-size: 1.1rem; transition: background 0.2s;",
                                onclick: move |_| {
                                    gui_bridge::respond_incoming(&pending_id_decline, false);
                                },
                                "DECLINE"
                            }
                        }
                    }
                }
            }
        }
    }
}
