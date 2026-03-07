//! Dioxus GUI for FastShare

use crate::app::App as FastShareApp;
use dioxus::prelude::*;
use std::sync::Arc;

/// Main Dioxus application component.
pub fn app() -> Element {
    // Basic Dioxus desktop UI
    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; background-color: #1a1a2e; color: white; font-family: system-ui, sans-serif;",
            h1 {
                style: "font-size: 3rem; margin-bottom: 0.5rem; background: -webkit-linear-gradient(45deg, #FF6B6B, #4ECDC4); -webkit-background-clip: text; -webkit-text-fill-color: transparent;",
                "⚡ FastShare"
            }
            p {
                style: "font-size: 1.2rem; margin-bottom: 2rem; color: #a9b5c9;",
                "Ultra-High-Performance P2P File Transfer"
            }

            div {
                style: "display: flex; gap: 1rem; margin-top: 1rem;",
                button {
                    style: "padding: 0.8rem 2rem; font-size: 1rem; font-weight: bold; border-radius: 8px; background-color: #4ECDC4; color: #1a1a2e; border: none; cursor: pointer; transition: transform 0.2s;",
                    onmouseenter: move |event| { /* no-op for now, needs direct css for hover usually */ },
                    "Send File"
                }
                button {
                    style: "padding: 0.8rem 2rem; font-size: 1rem; font-weight: bold; border-radius: 8px; background-color: transparent; color: #4ECDC4; border: 2px solid #4ECDC4; cursor: pointer; transition: background-color 0.2s;",
                    "Receive File"
                }
            }

            div {
                style: "margin-top: 3rem; background-color: #16213e; padding: 1.5rem; border-radius: 12px; width: 80%; max-width: 600px;",
                h3 { style: "margin-top: 0;", "Nearby Devices" }
                p { style: "color: #a9b5c9; font-style: italic;", "Scanning for devices..." }
            }
        }
    }
}
