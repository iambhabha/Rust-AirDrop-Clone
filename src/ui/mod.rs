//! UI module declarations.
//!
//! The UI layer provides terminal-based interface components.
//! These modules can be adapted for Dioxus GUI when the
//! frontend is integrated.

pub mod devices;
#[cfg(not(target_os = "android"))]
pub mod gui;
pub mod gui_bridge;
pub mod history;
pub mod home;
pub mod receive;
pub mod send;
pub mod transfer;
