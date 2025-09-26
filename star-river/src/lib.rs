pub mod api;
pub mod routes;
pub mod sse;
pub mod star_river;
pub mod websocket;

use crate::routes::create_app_routes;
use crate::star_river::StarRiver;
use crate::star_river::init_app;
use axum::extract::State;
use axum::http::HeaderValue;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use time::UtcOffset;
use time::macros::format_description;
use tokio;
use tower_http::cors::{Any, CorsLayer};
use tracing::instrument;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // 设置生产环境的日志级别
    // tracing_subscriber::fmt()
    //     // filter spans/events with level TRACE or higher.
    //     .with_max_level(Level::DEBUG)
    //     // build but do not install the subscriber.
    //     .init();
    // 确保log目录存在
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        fs::create_dir_all(log_dir)?;
    }
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "star-river.log");
    // 处理非阻塞appender
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);
    let stdout = std::io::stdout.with_max_level(tracing::Level::DEBUG);
    let filter = EnvFilter::new("debug,hyper=error,hyper_util=error,reqwest=error");

    // 设置为UTC+8时区（北京时间）
    let offset = UtcOffset::current_local_offset().expect("should get local offset!");
    let time_format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]");
    let timer = OffsetTime::new(offset, time_format);
    let console_layer = layer()
        .with_writer(stdout)
        .with_ansi(true) // 控制台保留ANSI颜色
        .with_timer(timer.clone());

    let file_layer = layer()
        .with_writer(non_blocking_appender)
        .with_ansi(false) // 文件中不使用ANSI颜色
        .with_timer(timer.clone());

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer) // 文件输出放到控制台输出的上方。不然文件中会有乱码
        .with(console_layer)
        .init();

    // tracing_subscriber::fmt()
    //     // filter spans/events with level TRACE or higher.
    //     .with_max_level(Level::DEBUG)
    //     .with_env_filter(filter)
    //     .with_timer(timer)
    //     .with_writer(stdout.and(file_appender))
    //     // build but do not install the subscriber.
    //     .init();

    // build our application with a route
    // 设置跨域
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);

    // 创建app状态
    let star_river = StarRiver::new().await;

    let app = create_app_routes(star_river.clone()).layer(cors);

    // 初始化app
    init_app(State(star_river)).await;

    // 允许从环境变量配置监听地址
    let addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3100".to_string())
        .parse::<SocketAddr>()
        .expect("Invalid server address");

    // run it
    // let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let listener = bind_with_retry(addr, 3).await?;

    #[cfg(windows)]
    {
        clean_mt5_server()?
    }

    clean_mei_temp_dirs(); // 清理MetaTrader5的_MEI临时文件夹
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    print_startup_info(addr);
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let server = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>());
    let graceful = server.with_graceful_shutdown(async {
        rx.await.ok();
        tracing::info!("开始执行优雅关闭流程...");

        // 使用更短的超时时间包装清理流程
        // let cleanup_result = tokio::time::timeout(
        //     tokio::time::Duration::from_secs(3),
        //     async {
        // #[cfg(windows)]
        // {
        //     tracing::info!("正在清理 MetaTrader5 进程...");

        //     // 使用异步方式执行 taskkill 命令
        //     let result = tokio::process::Command::new("taskkill")
        //         .args(&["/F", "/IM", "MetaTrader5.exe"])
        //         .output()
        //         .await;

        //     match result {
        //         Ok(output) => {
        //             tracing::info!("清理 MetaTrader5 进程结果: 退出码={}, stdout={}, stderr={}",
        //                 output.status.code().unwrap_or(-1),
        //                 String::from_utf8_lossy(&output.stdout),
        //                 String::from_utf8_lossy(&output.stderr)
        //             );
        //         }
        //         Err(e) => {
        //             tracing::error!("执行 taskkill 命令失败: {}", e);
        //         }
        //     }

        //     // 等待进程完全清理
        //     tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        //     tracing::info!("MetaTrader5 进程清理完成");
        // }

        //         #[cfg(not(windows))]
        //         {
        //             tracing::info!("非 Windows 系统，跳过 MetaTrader5 清理");
        //         }
        //     }
        // ).await;

        // match cleanup_result {
        //     Ok(_) => {
        //         tracing::info!("清理完成，继续关闭服务器...");
        //     }
        //     Err(_) => {
        //         tracing::warn!("清理流程超时，但继续关闭服务器...");
        //     }
        // }

        tracing::info!("优雅关闭流程完成，等待服务器停止...");
    });

    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            tracing::info!("接收到关闭信号，正在优雅关闭...");

            // 启动强制退出保护机制，只有在接收到关闭信号后才开始计时
            tokio::spawn(async {
                tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                tracing::error!("服务器关闭流程超时（15秒），强制退出...");
                std::process::exit(1);
            });

            let _ = tx.send(());
        }
    });

    // 直接等待服务器关闭，不设置外层超时
    if let Err(e) = graceful.await {
        tracing::error!("服务器错误: {}", e);
    } else {
        tracing::info!("服务器已成功关闭");
    }

    Ok(())
}

async fn bind_with_retry(addr: SocketAddr, max_retries: u32) -> Result<tokio::net::TcpListener, Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                if retries >= max_retries {
                    return Err(format!("端口 {} 被占用，重试 {} 次后仍然失败", addr.port(), max_retries).into());
                }
                tracing::warn!("端口 {} 被占用，尝试清理所有 StarRiver 相关进程...", addr.port());

                // 等待进程完全退出
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                retries += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

fn clean_mt5_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing::debug!("start cleaning MT5 server");
    // 1. 首先检查并清理原始的 MetaTrader5.exe 进程
    // 完整命令: tasklist /FI "IMAGENAME eq MetaTrader5.exe" /FO CSV
    let output = std::process::Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq MetaTrader5.exe", "/FO", "CSV"])
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    if output_str.contains("MetaTrader5.exe") {
        tracing::warn!("发现旧的MetaTrader5.exe进程, 正在清理...");

        // 完整命令: taskkill /F /IM MetaTrader5.exe
        let kill_result = std::process::Command::new("taskkill")
            .args(&["/F", "/IM", "MetaTrader5.exe"])
            .output();

        match kill_result {
            Ok(_) => tracing::info!("成功清理 MetaTrader5.exe 进程"),
            Err(e) => tracing::warn!("清理 MetaTrader5.exe 进程失败: {}", e),
        }
    }

    // 2. 检查并清理带有数字后缀的 Metatrader5-*.exe 进程
    // 使用tasklist命令查找所有进程，然后筛选Metatrader5-*进程（兼容老旧机型）
    // 完整命令: tasklist /FO CSV
    let output = std::process::Command::new("tasklist").args(&["/FO", "CSV"]).output();

    match output {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();
            let mut found_processes = Vec::new();

            // 查找所有Metatrader5-*进程
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

                // 逐个清理找到的进程
                for process_name in found_processes {
                    // 完整命令: taskkill /F /IM <process_name>
                    let kill_result = std::process::Command::new("taskkill").args(&["/F", "/IM", &process_name]).output();

                    match kill_result {
                        Ok(_) => tracing::info!("成功清理进程: {}", process_name),
                        Err(e) => tracing::warn!("清理进程 {} 失败: {}", process_name, e),
                    }
                }
            }
        }
        Err(e) => tracing::warn!("检查 Metatrader5-*.exe 进程失败: {}", e),
    }

    // 3. 如果上面的通配符方法不起作用，可以尝试列出所有进程并逐一匹配
    // 完整命令: tasklist /FO CSV
    let output = std::process::Command::new("tasklist").args(&["/FO", "CSV"]).output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();

    for line in lines {
        if line.contains("Metatrader5-") {
            // 从行中提取进程名称
            if let Some(process_name) = line.split(',').nth(0) {
                let process_name = process_name.trim_matches('"');
                tracing::warn!("发现MetaTrader5相关进程: {}, 正在清理...", process_name);

                // 完整命令: taskkill /F /IM <process_name>
                let kill_result = std::process::Command::new("taskkill").args(&["/F", "/IM", process_name]).output();

                match kill_result {
                    Ok(_) => {
                        tracing::info!("成功清理进程: {}", process_name)
                    }
                    Err(e) => {
                        tracing::warn!("清理进程 {} 失败: {}", process_name, e);
                    }
                }
            }
        }
    }

    Ok(())
}

// 清理MetaTrader5的临时文件夹
fn clean_mei_temp_dirs() {
    // 获取临时目录
    if let Ok(temp_dir) = std::env::var("TEMP").or_else(|_| std::env::var("TMP")) {
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with("_MEI") {
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
    }
}

/// 打印服务启动信息和文档链接（简洁版）
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
