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

#[component]
fn DeviceButton(
    device: DeviceInfo,
    selected_device: Signal<Option<DeviceInfo>>,
    selected_files: Signal<Vec<PathBuf>>,
) -> Element {
    let dev = device.clone();
    let is_selected = selected_device()
        .as_ref()
        .map_or(false, |d| d.device_id == device.device_id);
    let bg_color = if is_selected {
        "rgba(255,255,255,0.05)"
    } else {
        "transparent"
    };

    // Choose icon
    let icon_char = if dev.device_type.to_lowercase().contains("android")
        || dev.device_type.to_lowercase().contains("phone")
    {
        "📱"
    } else if dev.device_type.to_lowercase().contains("mac")
        || dev.device_type.to_lowercase().contains("windows")
        || dev.device_type.to_lowercase().contains("pc")
        || dev.device_type.to_lowercase().contains("desktop")
        || dev.device_type.to_lowercase().contains("laptop")
    {
        "💻"
    } else {
        "💻"
    };

    rsx! {
        button {
            style: "width: 100%; display: flex; align-items: center; gap: 1.2rem; padding: 0.9rem; background-color: {bg_color}; border: none; border-radius: 8px; cursor: pointer; transition: background 0.2s;",
            onclick: move |_| {
                selected_device.set(Some(dev.clone()));
                // Immediately open file picker and send
                let mut sel_files = selected_files;
                let device_clone = dev.clone();
                spawn(async move {
                    let files = rfd::AsyncFileDialog::new().pick_files().await;
                    if let Some(paths) = files {
                        let list: std::vec::Vec<std::path::PathBuf> = paths.into_iter().map(|f| f.path().to_path_buf()).collect();
                        if !list.is_empty() {
                            if let Some(b) = gui_bridge::get_bridge() {
                                let addr = std::net::SocketAddr::new(device_clone.ip_address, device_clone.port);
                                let _ = b.send_tx.try_send((list, addr));
                            }
                        }
                    }
                });
            },
            div {
                style: "width: 44px; height: 44px; border-radius: 50%; background-color: #e87d65; display: flex; align-items: center; justify-content: center; font-size: 1.4rem; color: white; flex-shrink: 0;",
                "{icon_char}"
            }
            div {
                style: "display: flex; flex-direction: column; align-items: flex-start; text-align: left; overflow: hidden; white-space: nowrap;",
                span {
                    style: "color: white; font-weight: 500; font-size: 1.05rem; text-overflow: ellipsis; overflow: hidden; max-width: 100%; letter-spacing: 0.2px;",
                    "{device.device_name}"
                }
                span {
                    style: "color: #888; font-size: 0.85rem; text-overflow: ellipsis; overflow: hidden; max-width: 100%; margin-top: 0.2rem;",
                    "IP: {device.ip_address}"
                }
            }
        }
    }
}

/// Main Dioxus application component.
pub fn app() -> Element {
    let mut selected_files = use_signal(|| Vec::<PathBuf>::new());
    let mut selected_device = use_signal(|| None::<DeviceInfo>);
    let mut status_message = use_signal(|| None::<String>);
    let mut show_settings = use_signal(|| false);
    let refresh_tick = use_signal(|| 0u32);

    let transfer_progress_signal = use_signal(|| None::<crate::transfer::sender::TransferProgress>);
    let incoming_progress_signal =
        use_signal(|| Vec::<crate::ui::gui_bridge::IncomingProgress>::new());

    // Poll every 1.5s
    use_effect(move || {
        let mut tick = refresh_tick;
        spawn(async move {
            for _ in 0..1000 {
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                tick.set(tick() + 1);
            }
        });
    });

    // Poll progress every 200ms
    use_effect(move || {
        let mut stat = status_message;
        let mut tp = transfer_progress_signal;
        let mut ip = incoming_progress_signal;
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                tp.set(gui_bridge::get_transfer_progress());
                ip.set(gui_bridge::get_incoming_progress());

                if let Some(msg) = gui_bridge::take_backend_status() {
                    stat.set(Some(msg));
                    spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        stat.set(None);
                    });
                }
                if let Some(prog) = gui_bridge::get_transfer_progress() {
                    if prog.complete {
                        spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                            gui_bridge::clear_transfer_progress();
                        });
                    }
                }
            }
        });
    });

    let _refresh_eval = refresh_tick();
    let transfer_progress = transfer_progress_signal();
    let incoming_progress = incoming_progress_signal();

    let bridge = gui_bridge::get_bridge();
    let mut devices: Vec<DeviceInfo> = if let Some(ref b) = bridge {
        match b.state.nearby_devices.try_read() {
            Ok(guard) => guard.clone(),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };
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
    let (
        pending_file_id,
        pending_from,
        pending_fname,
        pending_total_files,
        _pending_total_size,
        _pending_batch_size,
    ) = pending
        .as_ref()
        .map(|p| (p.0.clone(), format!("{}", p.1), p.2.clone(), p.3, p.4, p.5))
        .unwrap_or((String::new(), String::new(), String::new(), 1, 0, 0));
    let show_incoming = pending.is_some();
    let pending_id_accept = pending_file_id.clone();
    let pending_id_decline = pending_file_id.clone();

    rsx! {
        div {
            style: "display: flex; height: 100vh; width: 100vw; overflow: hidden; background-color: #1c1c1c; color: white; font-family: 'Ligurino', system-ui, sans-serif; user-select: none;",

            // Sidebar
            div {
                style: "width: 320px; background-color: #262626; display: flex; flex-direction: column; flex-shrink: 0;",

                // Header
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 2.2rem 1.6rem 1.2rem 1.6rem;",
                    div {
                        style: "display: flex; align-items: center; gap: 0.3rem;",
                        h1 {
                            style: "margin: 0; font-size: 1.9rem; font-weight: 700; letter-spacing: -0.3px; color: #ffffff;",
                            "Rust Drop"
                        }
                    }
                    button {
                        style: "background: none; border: none; color: #a0a0a0; cursor: pointer; font-size: 1.4rem; padding: 0; display: flex; align-items: center; justify-content: center; transition: color 0.2s;",
                        onclick: move |_| {
                            let mut s = show_settings;
                            s.set(!s());
                        },
                        dangerous_inner_html: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>"#
                    }

                    if show_settings() {
                        div {
                            style: "position: absolute; top: 4.8rem; left: 160px; width: 220px; background-color: #2b2b2b; border: 1px solid #3d3d3d; border-radius: 12px; box-shadow: 0 15px 40px rgba(0,0,0,0.6); z-index: 1000; display: flex; flex-direction: column; overflow: hidden;",

                            // User Profile
                            div {
                                style: "display: flex; align-items: center; gap: 1rem; padding: 1.2rem; border-bottom: 1px solid #3d3d3d;",
                                div {
                                    style: "width: 36px; height: 36px; border-radius: 50%; background-color: #555; display: flex; align-items: center; justify-content: center; font-weight: 500; font-size: 1.1rem; color: #fff;",
                                    "D"
                                }
                                div {
                                    style: "display: flex; flex-direction: column; margin-top: -2px;",
                                    span { style: "color: white; font-weight: 500; font-size: 1.05rem;", "dev" }
                                    span { style: "color: #a0a0a0; font-size: 0.8rem; margin-top: 1px;", "devrajheropanti@gmail.co..." }
                                }
                            }

                            // Feedback & Help
                            div {
                                style: "padding: 1rem 1.2rem; border-bottom: 1px solid #3d3d3d;",
                                p { style: "margin: 0 0 0.8rem 0; color: #a0a0a0; font-size: 0.8rem; font-weight: 500;", "Feedback & Help" }
                                div { style: "display: flex; align-items: center; gap: 1rem; color: #e5e5e5; padding: 0.45rem 0; cursor: pointer; font-size: 0.95rem; font-weight: 500;",
                                    span { style: "color: #d0d0d0; display: flex;", dangerous_inner_html: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path><polyline points="22,6 12,13 2,6"></polyline></svg>"# }
                                    "Email"
                                }
                                div { style: "display: flex; align-items: center; gap: 1rem; color: #e5e5e5; padding: 0.45rem 0; cursor: pointer; font-size: 0.95rem; font-weight: 500;",
                                    span { style: "color: #d0d0d0; display: flex;", dangerous_inner_html: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>"# }
                                    "Discord"
                                }
                                div { style: "display: flex; align-items: center; gap: 1rem; color: #e5e5e5; padding: 0.45rem 0; cursor: pointer; font-size: 0.95rem; font-weight: 500;",
                                    span { style: "color: #d0d0d0; display: flex;", dangerous_inner_html: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M23 3a10.9 10.9 0 0 1-3.14 1.53 4.48 4.48 0 0 0-7.86 3v1A10.66 10.66 0 0 1 3 4s-4 9 5 13a11.64 11.64 0 0 1-7 2c9 5 20 0 20-11.5a4.5 4.5 0 0 0-.08-.83A7.72 7.72 0 0 0 23 3z"></path></svg>"# }
                                    "Twitter"
                                }
                            }

                            // Support Blip
                            div {
                                style: "padding: 1rem 1.2rem;",
                                p { style: "margin: 0 0 0.8rem 0; color: #a0a0a0; font-size: 0.8rem; font-weight: 500;", "Support Blip" }
                                div { style: "display: flex; align-items: center; gap: 1rem; color: #e5e5e5; padding: 0.45rem 0; cursor: pointer; font-size: 0.95rem; font-weight: 500;",
                                    span { style: "color: #d0d0d0; display: flex;", dangerous_inner_html: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"></path></svg>"# }
                                    "Donate"
                                }
                                div { style: "display: flex; align-items: center; gap: 1rem; color: #e5e5e5; padding: 0.45rem 0; cursor: pointer; font-size: 0.95rem; font-weight: 500;",
                                    span { style: "color: #d0d0d0; display: flex;", dangerous_inner_html: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="5" width="20" height="14" rx="2" ry="2"></rect><line x1="2" y1="10" x2="22" y2="10"></line></svg>"# }
                                    "Upgrade"
                                }
                                div { style: "display: flex; align-items: center; justify-content: space-between; color: #e5e5e5; padding: 0.45rem 0; cursor: pointer; font-size: 0.95rem; font-weight: 500; margin-top: 0.5rem;",
                                    div { style: "display: flex; align-items: center; gap: 1rem;",
                                        span { style: "color: #d0d0d0; display: flex;", dangerous_inner_html: r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>"# }
                                        "Settings"
                                    }
                                    span { style: "color: #777; font-size: 0.75rem;", "v1.1.15" }
                                }
                            }
                        }
                    }
                }

                // Search Input
                div {
                    style: "padding: 0 1.6rem 2rem 1.6rem;",
                    div {
                        style: "display: flex; align-items: center; background-color: #333333; border: 1px solid #3d3d3d; border-radius: 8px; padding: 0.8rem 1rem;",
                        input {
                            r#type: "text",
                            placeholder: "Name or email",
                            style: "background: transparent; border: none; color: white; outline: none; width: 100%; font-size: 0.95rem; font-family: inherit;",
                        }
                        span {
                            // Easily replaceable Search Icon
                            style: "color: #777; margin-left: 0.5rem; display: flex; align-items: center; justify-content: center;",
                            dangerous_inner_html: r#"<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>"#
                        }
                    }
                }

                // Device List
                div {
                    style: "flex: 1; overflow-y: auto; padding: 0 1.0rem 1.5rem 1.0rem; display: flex; flex-direction: column; gap: 0.2rem;",
                    for dinfo in &devices {
                        DeviceButton {
                            device: dinfo.clone(),
                            selected_device: selected_device,
                            selected_files: selected_files,
                        }
                    }
                    if devices.is_empty() {
                         div {
                             style: "text-align: center; color: #777; font-size: 0.95rem; padding-top: 2.5rem; font-style: italic;",
                             "Searching for devices..."
                         }
                    }
                }
            }

            // Main Content Area
            div {
                style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; position: relative; background-color: #1b1b1b;",

                if transfer_progress.is_none() && incoming_progress.is_empty() {
                    // Welcome Splash
                    div {
                        style: "text-align: center; display: flex; flex-direction: column; align-items: center;",
                        h2 {
                            style: "margin: 0; font-size: 2.1rem; font-weight: 700; color: rgba(255,255,255,0.3); letter-spacing: -0.2px;",
                            "Welcome to Rust Drop"
                        }
                        p {
                            style: "margin: 1.2rem 0 0; font-size: 1.15rem; color: #666; text-align: center; max-width: 380px; line-height: 1.5; font-weight: 400;",
                            "Send any size file or folder to anyone, wherever they are, super fast"
                        }
                    }
                } else {
                    // Transfer Progress View
                    div {
                        style: "width: 100%; max-width: 550px; padding: 2rem; display: flex; flex-direction: column; gap: 1.5rem;",

                        if let Some(ref prog) = transfer_progress {
                            div {
                                style: "background: #2b2b2b; padding: 1.8rem; border-radius: 12px; border: 1px solid #3d3d3d;",
                                h4 { style: "margin: 0 0 1.2rem; color: #e87d65; font-size: 1.15rem; font-weight: 600;", "📤 Sending..." }
                                p { style: "margin: 0 0 1.5rem; font-weight: 500; font-size: 1.1rem; color: white;", "{prog.file_name}" }
                                div {
                                    style: "width: 100%; height: 6px; background: #1e1e1e; border-radius: 3px; overflow: hidden; margin-bottom: 1rem;",
                                    div {
                                        style: "height: 100%; background: #e87d65; transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1); width: {format_size_pct(progress_pct(prog))};",
                                    }
                                }
                                div {
                                    style: "display: flex; justify-content: space-between; font-size: 0.95rem; color: #888;",
                                    span { "{format_size_pct(progress_pct(prog))} completed" }
                                    span { "{format_throughput(prog.throughput_bps)}" }
                                }
                            }
                        }

                        if !incoming_progress.is_empty() {
                            div {
                                style: "background: #2b2b2b; padding: 1.8rem; border-radius: 12px; border: 1px solid #3d3d3d;",
                                h4 { style: "margin: 0 0 1.2rem; color: #4ECDC4; font-size: 1.15rem; font-weight: 600;", "📥 Receiving..." }
                                div {
                                    style: "display: flex; flex-direction: column; gap: 1.5rem;",
                                    {
                                        incoming_progress.into_iter().map(|ip| {
                                            let pct = format_size_pct(ip.progress * 100.0);
                                            let fname = ip.file_name.clone();
                                            rsx! {
                                                div {
                                                    key: "{fname}",
                                                    p { style: "margin: 0 0 1rem; font-weight: 500; font-size: 1.1rem; color: white;", "{fname}" }
                                                    div {
                                                        style: "width: 100%; height: 6px; background: #1e1e1e; border-radius: 3px; overflow: hidden; margin-bottom: 1rem;",
                                                        div {
                                                            style: "height: 100%; background: #4ECDC4; transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1); width: {pct};",
                                                        }
                                                    }
                                                    div { style: "display: flex; justify-content: space-between; font-size: 0.95rem; color: #888;",
                                                        span { "{pct} completed" }
                                                        span { "{format_size(ip.received_bytes)} / {format_size(ip.total_bytes)}" }
                                                    }
                                                }
                                            }
                                        })
                                    }
                                }
                            }
                        }
                    }
                }

                // Snackbar
                if let Some(ref msg) = status_message() {
                    div {
                        style: "position: absolute; bottom: 2rem; background: #e87d65; color: white; padding: 1rem 1.8rem; border-radius: 30px; font-weight: 500; font-size: 1rem; box-shadow: 0 8px 25px rgba(0,0,0,0.4); z-index: 2000;",
                        "✨ {msg}"
                    }
                }
            }

            // Incoming modal
            if show_incoming {
                div {
                    style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 3000; backdrop-filter: blur(4px);",
                    div {
                        style: "background: #2b2b2b; padding: 2.5rem; border-radius: 16px; width: 90%; max-width: 420px; box-shadow: 0 15px 40px rgba(0,0,0,0.4); text-align: center; border: 1px solid #3d3d3d;",
                        h2 { style: "margin: 0 0 1rem; color: white; font-size: 1.6rem; font-weight: 600;", if pending_total_files > 1 { "Receive Files?" } else { "Receive File?" } }
                        div {
                            style: "margin: 1.5rem 0; padding: 1.5rem; background: #1e1e1e; border-radius: 12px; text-align: left; border: 1px solid #333;",
                            p { style: "color: #777; margin: 0; font-size: 0.9rem;", "From Device" }
                            p { style: "color: white; margin: 0.3rem 0 1.2rem; font-weight: 500; font-size: 1.1rem;", "{pending_from}" }
                            p { style: "color: #777; margin: 0; font-size: 0.9rem;", "File Name" }
                            p { style: "color: white; margin: 0.3rem 0; font-weight: 500; font-size: 1.05rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{pending_fname}" }
                        }
                        div {
                            style: "display: flex; gap: 1rem; margin-top: 2rem;",
                            button {
                                style: "flex: 1; padding: 0.9rem; border-radius: 8px; background: transparent; color: #a0a0a0; border: 1px solid #444; cursor: pointer; font-weight: 500; font-size: 1rem; transition: background 0.2s;",
                                onclick: move |_| {
                                    gui_bridge::respond_incoming(&pending_id_decline, false);
                                },
                                "Decline"
                            }
                            button {
                                style: "flex: 1; padding: 0.9rem; border-radius: 8px; background: #e87d65; color: white; border: none; cursor: pointer; font-weight: 500; font-size: 1rem; transition: background 0.2s;",
                                onclick: move |_| {
                                    gui_bridge::respond_incoming(&pending_id_accept, true);
                                },
                                "Accept"
                            }
                        }
                    }
                }
            }
        }
    }
}
