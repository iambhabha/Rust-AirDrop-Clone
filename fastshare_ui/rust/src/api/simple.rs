use fastshare::app::App;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::runtime::Runtime;

lazy_static::lazy_static! {
    static ref RUNTIME: Runtime = tokio::runtime::Runtime::new().unwrap();
}

pub struct ServerHandle {
    device_id: String,
    listen_addr: String,
}

#[flutter_rust_bridge::frb(sync)]
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

/// Start the background FastShare server and return some details
pub fn start_fastshare() -> String {
    let _ = RUNTIME.block_on(async {
        match App::new().await {
            Ok(app) => {
                let id = app.device_id().to_string();
                let addr = app.listen_addr().to_string();

                tokio::spawn(async move {
                    let _ = app.run().await;
                });

                format!("Started FastShare as {}\nListening on {}", id, addr)
            }
            Err(e) => format!("Failed: {}", e),
        }
    });
    "FastShare Backend Started!".into()
}
