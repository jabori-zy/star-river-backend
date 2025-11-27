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

/// 初始化日志系统
pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // 确保log目录存在
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        fs::create_dir_all(log_dir)?;
    }

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "star-river.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);
    let stdout = std::io::stdout.with_max_level(tracing::Level::DEBUG);
    let filter = EnvFilter::new("debug,hyper=error,hyper_util=error,reqwest=error");

    // 设置本地时区
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

/// 创建 CORS 配置
pub fn create_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any)
}

/// 启动服务器
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
        tracing::info!("开始执行优雅关闭流程...");
        tracing::info!("优雅关闭流程完成，等待服务器停止...");
    });

    // 处理 Ctrl+C 信号
    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            tracing::info!("接收到关闭信号，正在优雅关闭...");

            // 强制退出保护机制
            tokio::spawn(async {
                tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                tracing::error!("服务器关闭流程超时，强制退出...");
                std::process::exit(1);
            });

            let _ = tx.send(());
        }
    });

    // 等待服务器关闭
    if let Err(e) = graceful.await {
        tracing::error!("服务器错误: {}", e);
    } else {
        tracing::info!("服务器已成功关闭");
    }

    Ok(())
}

/// 重试绑定端口
async fn bind_with_retry(addr: SocketAddr, max_retries: u32) -> Result<tokio::net::TcpListener, Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                if retries >= max_retries {
                    return Err(format!("端口 {} 被占用，重试 {} 次后仍然失败", addr.port(), max_retries).into());
                }
                tracing::warn!("端口 {} 被占用，等待重试...", addr.port());
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                retries += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

/// 清理 MetaTrader5 进程 (仅 Windows)
#[cfg(windows)]
fn clean_mt5_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!("start cleaning MT5 server");

    // 1. 清理原始的 MetaTrader5.exe 进程
    let output = std::process::Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq MetaTrader5.exe", "/FO", "CSV"])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    if output_str.contains("MetaTrader5.exe") {
        tracing::warn!("发现旧的MetaTrader5.exe进程, 正在清理...");
        let kill_result = std::process::Command::new("taskkill")
            .args(&["/F", "/IM", "MetaTrader5.exe"])
            .output();

        match kill_result {
            Ok(_) => tracing::info!("成功清理 MetaTrader5.exe 进程"),
            Err(e) => tracing::warn!("清理 MetaTrader5.exe 进程失败: {}", e),
        }
    }

    // 2. 清理带数字后缀的 Metatrader5-*.exe 进程
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
            tracing::warn!("发现Metatrader5-*.exe进程: {:?}, 正在清理...", found_processes);
            for process_name in found_processes {
                let _ = std::process::Command::new("taskkill").args(&["/F", "/IM", &process_name]).output();
            }
        }
    }

    Ok(())
}

/// 清理 MetaTrader5 的临时文件夹
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
                        Ok(_) => tracing::info!("已删除_MEI临时文件夹: {}", path.display()),
                        Err(e) => tracing::warn!("删除_MEI临时文件夹失败: {}, 错误: {}", path.display(), e),
                    }
                }
            }
        }
    }
}

/// 打印服务启动信息
#[instrument]
fn print_startup_info(addr: SocketAddr) {
    let host = if addr.ip().is_unspecified() { "localhost" } else { "localhost" };
    let port = addr.port();
    let base_url = format!("http://{}:{}", host, port);

    tracing::info!("🚀 Star River 启动成功!");
    tracing::info!("📡 服务地址: {}", addr);
    tracing::info!("📚 API 文档: {}/docs", base_url);
    tracing::info!("🔗 OpenAPI:  {}/api-docs/openapi.json", base_url);
    tracing::info!("按 Ctrl+C 停止服务\n");
}
