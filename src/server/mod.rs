pub mod sse;

use std::path::PathBuf;
use std::time::Duration;

use axum::Router;
use notify::{RecursiveMode, Watcher};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tracing::{error, info};

use crate::build;
use crate::error::{Result, TypsyError};

/// Run the development server: initial build, file watcher, axum with SSE live reload.
pub async fn run_dev_server(root: PathBuf, port: u16) -> Result<()> {
    // Initial build (synchronous, run on blocking thread)
    let build_root = root.clone();
    tokio::task::spawn_blocking(move || build::build(&build_root, true))
        .await
        .unwrap();

    // Broadcast channel for SSE reload signals
    let (tx, _rx) = broadcast::channel::<()>(16);

    // Spawn file watcher on a dedicated OS thread
    spawn_watcher(root.clone(), tx.clone());

    // Build axum router
    let out_dir = root.join("out");
    let app = Router::new()
        .route(
            "/__typsy_reload",
            axum::routing::get(sse::reload_handler),
        )
        .fallback_service(ServeDir::new(&out_dir).append_index_html_on_directories(true))
        .with_state(sse::ReloadState { tx });

    let addr = format!("0.0.0.0:{port}");
    info!("serving at http://localhost:{port}");
    info!("watching for changes... (ctrl+c to stop)");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| TypsyError::Io {
            path: PathBuf::from(&addr),
            source: e,
        })?;

    axum::serve(listener, app)
        .await
        .map_err(|e| TypsyError::Io {
            path: PathBuf::from("server"),
            source: e,
        })?;

    Ok(())
}

/// Spawn a file watcher on a dedicated OS thread.
///
/// Watches `content/`, `static/`, and `lib/` directories. On changes, debounces
/// for 100ms, rebuilds the site, then notifies SSE clients to reload.
fn spawn_watcher(root: PathBuf, tx: broadcast::Sender<()>) {
    std::thread::spawn(move || {
        let (file_tx, file_rx) = std::sync::mpsc::channel();

        let mut watcher =
            notify::recommended_watcher(move |res: std::result::Result<notify::Event, _>| {
                if res.is_ok() {
                    file_tx.send(()).ok();
                }
            })
            .expect("failed to create file watcher");

        for dir in ["content", "static", "lib"] {
            let path = root.join(dir);
            if path.exists() {
                watcher.watch(&path, RecursiveMode::Recursive).ok();
            }
        }

        loop {
            if file_rx.recv().is_ok() {
                // Debounce: drain events for 100ms
                while file_rx.recv_timeout(Duration::from_millis(100)).is_ok() {}

                info!("changes detected, rebuilding...");
                let report = build::build(&root, true);

                for failure in &report.failures {
                    error!("{failure}");
                }

                // Notify all connected browsers to reload
                let _ = tx.send(());
            }
        }
    });
}
