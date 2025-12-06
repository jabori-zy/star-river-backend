use std::{fs, net::SocketAddr, path::Path};

use axum::{Router, http::HeaderValue};
use time::{UtcOffset, macros::format_description};
use tower_http::cors::{Any, CorsLayer};
use tracing::instrument;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter,
    fmt::{layer, time::OffsetTime, writer::MakeWriterExt},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Initialize logging system
pub fn init_logging(stdout_level: tracing::Level) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure log directory exists
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        fs::create_dir_all(log_dir)?;
    }

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "star-river.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);
    let stdout = std::io::stdout.with_max_level(stdout_level);
    let filter = EnvFilter::new("debug,hyper=error,hyper_util=error,reqwest=error");

    // Set local timezone
    let offset = UtcOffset::current_local_offset().expect("should get local offset!");
    let time_format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]");
    let timer = OffsetTime::new(offset, time_format);

    let console_layer = layer().with_writer(stdout).with_ansi(true).with_timer(timer.clone());

    let file_layer = layer()
        .with_writer(non_blocking_appender)
        .with_ansi(false)
        .with_timer(timer.clone());

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    Ok(())
}

/// Create CORS configuration
pub fn create_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any)
}

/// Start server
pub async fn serve(app: Router, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let listener = bind_with_retry(addr, 3).await?;

    #[cfg(windows)]
    {
        clean_mt5_server()?;
    }

    clean_mei_temp_dirs();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    print_startup_info(addr);

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let server = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>());
    let graceful = server.with_graceful_shutdown(async {
        rx.await.ok();
        tracing::info!("Starting graceful shutdown process...");
        tracing::info!("Graceful shutdown process completed, waiting for server to stop...");
    });

    // Handle Ctrl+C signal
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            tracing::info!("Received shutdown signal, gracefully shutting down...");

            // Force exit protection mechanism
            tokio::spawn(async {
                tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                tracing::error!("Server shutdown process timed out, forcing exit...");
                std::process::exit(1);
            });

            let _ = tx.send(());
        }
    });

    // Wait for server shutdown
    if let Err(e) = graceful.await {
        tracing::error!("Server error: {}", e);
    } else {
        tracing::info!("Server successfully shut down");
    }

    Ok(())
}

/// Retry binding port
async fn bind_with_retry(addr: SocketAddr, max_retries: u32) -> Result<tokio::net::TcpListener, Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                if retries >= max_retries {
                    return Err(format!("Port {} is in use, failed after {} retry attempts", addr.port(), max_retries).into());
                }
                tracing::warn!("Port {} is in use, waiting to retry...", addr.port());
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                retries += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

/// Clean up MetaTrader5 processes (Windows only)
#[cfg(windows)]
fn clean_mt5_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!("start cleaning MT5 server");

    // 1. Clean up original MetaTrader5.exe processes
    let output = std::process::Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq MetaTrader5.exe", "/FO", "CSV"])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    if output_str.contains("MetaTrader5.exe") {
        tracing::warn!("Found old MetaTrader5.exe process, cleaning up...");
        let kill_result = std::process::Command::new("taskkill")
            .args(&["/F", "/IM", "MetaTrader5.exe"])
            .output();

        match kill_result {
            Ok(_) => tracing::info!("Successfully cleaned up MetaTrader5.exe process"),
            Err(e) => tracing::warn!("Failed to clean up MetaTrader5.exe process: {}", e),
        }
    }

    // 2. Clean up Metatrader5-*.exe processes with numeric suffixes
    let output = std::process::Command::new("tasklist").args(&["/FO", "CSV"]).output();

    if let Ok(output) = output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        let mut found_processes = Vec::new();

        for line in lines {
            if line.contains("Metatrader5-") && line.contains(".exe") {
                if let Some(process_name) = line.split(',').nth(0) {
                    let process_name = process_name.trim_matches('"');
                    if process_name.starts_with("Metatrader5-") && process_name.ends_with(".exe") {
                        found_processes.push(process_name.to_string());
                    }
                }
            }
        }

        if !found_processes.is_empty() {
            tracing::warn!("Found Metatrader5-*.exe processes: {:?}, cleaning up...", found_processes);
            for process_name in found_processes {
                let _ = std::process::Command::new("taskkill").args(&["/F", "/IM", &process_name]).output();
            }
        }
    }

    Ok(())
}

/// Clean up MetaTrader5 temporary folders
fn clean_mei_temp_dirs() {
    if let Ok(temp_dir) = std::env::var("TEMP").or_else(|_| std::env::var("TMP"))
        && let Ok(entries) = std::fs::read_dir(&temp_dir)
    {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string()
                && file_name.starts_with("_MEI")
            {
                let path = entry.path();
                if path.is_dir() {
                    match std::fs::remove_dir_all(&path) {
                        Ok(_) => tracing::info!("Deleted _MEI temporary folder: {}", path.display()),
                        Err(e) => tracing::warn!("Failed to delete _MEI temporary folder: {}, error: {}", path.display(), e),
                    }
                }
            }
        }
    }
}

/// Print server startup information
#[instrument]
fn print_startup_info(addr: SocketAddr) {
    let host = if addr.ip().is_unspecified() { "localhost" } else { "localhost" };
    let port = addr.port();
    let base_url = format!("http://{}:{}", host, port);

    tracing::info!("🚀 Star River Server");
    tracing::info!("📡 Server address: {}", addr);
    tracing::info!("📚 API documentation: {}/docs", base_url);
    tracing::info!("🔗 OpenAPI:  {}/api-docs/openapi.json", base_url);
    tracing::info!("Press Ctrl+C to stop the service\n");
}
