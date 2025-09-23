mod mt5_http_client;

mod mt5_data_processor;
mod mt5_types;
mod mt5_ws_client;
#[cfg(test)]
mod test;
mod url;

use super::metatrader5::mt5_types::Mt5CreateOrderParams;
use super::metatrader5::mt5_types::Mt5KlineInterval;
use crate::ExchangeClient;
use async_trait::async_trait;
use event_center::EventPublisher;
use futures::SinkExt;
use futures::StreamExt;
use mt5_data_processor::Mt5DataProcessor;
use mt5_http_client::Mt5HttpClient;
use mt5_types::Mt5GetPositionNumberParams;
use mt5_ws_client::Mt5WsClient;
use mt5_ws_client::WebSocketState;
use once_cell::sync::Lazy;
use rust_embed::Embed;
use serde_json::json;
use snafu::OptionExt;
use snafu::ResultExt;
use star_river_core::account::OriginalAccountInfo;
use star_river_core::error::exchange_client_error::*;
use star_river_core::market::KlineInterval;
use star_river_core::market::MT5Server;
use star_river_core::market::Symbol;
use star_river_core::market::{Exchange, Kline};
use star_river_core::order::{CreateOrderParams, GetTransactionDetailParams};
use star_river_core::order::{Order, OriginalOrder};
use star_river_core::position::PositionNumber;
use star_river_core::position::{GetPositionNumberParams, GetPositionParam, OriginalPosition, Position};
use star_river_core::strategy::TimeRange;
use star_river_core::transaction::{OriginalTransaction, Transaction};
use std::any::Any;
use std::fs;
use std::os::windows::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tracing::instrument;
use windows::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;

#[derive(Embed)]
#[folder = "src/metatrader5/bin/windows/"]
struct Asset;

// 存储原始可执行文件的永久路径
static ORIGINAL_EXE_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let app_data = if let Ok(app_data) = std::env::var("APPDATA") {
        tracing::debug!(app_data = %app_data, "get appdata");
        PathBuf::from(app_data)
    } else {
        let temp_dir = std::env::temp_dir();
        tracing::debug!(temp_dir = ?temp_dir, "get temp dir");
        PathBuf::from(temp_dir)
    };
    let star_river_dir = app_data.join("StarRiver").join("MetaTrader5");

    // 确保目录存在
    if !star_river_dir.exists() {
        let _ = fs::create_dir_all(&star_river_dir);
    }

    // 原始exe文件的永久存储路径
    star_river_dir.join("MetaTrader5.exe")
});

// 从嵌入资源中提取原始exe文件，如果不存在或有更新
#[instrument]
fn ensure_original_exe_exists() -> Result<(), Mt5Error> {
    tracing::info!("ensure original executable file exists");
    let original_exe_path = ORIGINAL_EXE_PATH.as_path();
    let py_exe = Asset::get("MetaTrader5-x86_64-pc-windows-msvc.exe").context(OtherSnafu {
        message: "get python executable file failed".to_string(),
    })?;

    let needs_update = if !original_exe_path.exists() {
        true
    } else {
        // 可选：检查文件是否需要更新（如大小不同）
        match fs::metadata(original_exe_path) {
            Ok(metadata) => metadata.len() as usize != py_exe.data.len(),
            Err(_) => true,
        }
    };

    if needs_update {
        tracing::debug!(original_exe_path = %original_exe_path.display(), "create original executable file");
        fs::write(original_exe_path, py_exe.data).unwrap();
    }

    Ok(())
}

// 为特定终端创建唯一的exe副本，并清理旧文件
#[instrument]
fn create_terminal_exe(terminal_id: i32, process_name: &str) -> Result<PathBuf, Mt5Error> {
    tracing::info!(terminal_id = %terminal_id, process_name=%process_name, "start create terminal exe");
    let original_exe_path = ORIGINAL_EXE_PATH.as_path();

    // 为每个终端创建特定工作目录
    let app_data = if let Ok(app_data) = std::env::var("APPDATA") {
        PathBuf::from(app_data)
    } else {
        PathBuf::from(std::env::temp_dir())
    };

    let terminal_dir = app_data
        .join("StarRiver")
        .join("MetaTrader5")
        .join(format!("terminal_{}", terminal_id));

    if !terminal_dir.exists() {
        let _ = fs::create_dir_all(&terminal_dir);
    } else {
        // 清理目录中所有已存在的exe文件
        match fs::read_dir(&terminal_dir) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        // 检查是否是文件且扩展名是.exe
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "exe")
                            || path.to_string_lossy().contains(&process_name)
                        {
                            if let Err(e) = fs::remove_file(&path) {
                                tracing::warn!(path = %path.display(), "failed to delete old exe file, error: {}", e);
                            } else {
                                tracing::debug!(path = %path.display(), "deleted old exe file");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to read terminal directory");
            }
        }
    }

    // 创建固定名称的exe文件，不使用时间戳
    let terminal_exe_path = terminal_dir.join(process_name);

    // 复制原始exe到新位置
    fs::copy(original_exe_path, &terminal_exe_path).unwrap();

    tracing::info!(terminal_id = %terminal_id, terminal_exe_path = %terminal_exe_path.display(), "create new exe file");

    Ok(terminal_exe_path)
}

#[derive(Clone, Debug)]
pub struct MetaTrader5 {
    pub process_name: String, // 进程名称
    pub server_port: u16,     // 服务器端口
    pub terminal_id: i32,     // 终端id
    pub login: i64,
    pub password: String,
    pub server: MT5Server,
    pub terminal_path: String,
    mt5_http_client: Arc<Mutex<Option<Mt5HttpClient>>>,
    websocket_state: Arc<Mutex<Option<WebSocketState>>>,
    data_processor: Arc<Mutex<Mt5DataProcessor>>,
    is_process_stream: Arc<AtomicBool>,
    // event_publisher: Arc<Mutex<EventPublisher>>,
    mt5_process: Arc<Mutex<Option<Child>>>,
    exe_path: Arc<Mutex<Option<PathBuf>>>,
}

impl MetaTrader5 {
    pub fn new(
        terminal_id: i32,
        login: i64,
        password: String,
        server: String,
        terminal_path: String,
        // event_publisher: EventPublisher
    ) -> Self {
        // let event_publisher = Arc::new(Mutex::new(event_publisher));

        Self {
            process_name: format!("Metatrader5-{}.exe", terminal_id),
            server_port: 8000 + terminal_id as u16,
            terminal_id,
            login,
            password,
            server: server.clone(),
            terminal_path,
            mt5_http_client: Arc::new(Mutex::new(None)),
            websocket_state: Arc::new(Mutex::new(None)),
            is_process_stream: Arc::new(AtomicBool::new(false)),
            // event_publisher: event_publisher.clone(),
            data_processor: Arc::new(Mutex::new(Mt5DataProcessor::new(server))),
            mt5_process: Arc::new(Mutex::new(None)),
            exe_path: Arc::new(Mutex::new(None)),
        }
    }

    // pub async fn start_mt5_server(&mut self, debug_output: bool) -> Result<u16, Mt5Error> {
    //     // 确保原始exe文件存在（只需执行一次）
    //     ensure_original_exe_exists()?;

    //     // 变量用于存储进程ID以便后续检查
    //     let mut old_pid: Option<u32> = None;

    //     // 先清理可能存在的旧进程
    //     let mt5_process = self.mt5_process.lock().await;
    //     if let Some(pid) = mt5_process.as_ref().and_then(|child| child.id()) {
    //         old_pid = Some(pid);
    //         #[cfg(windows)]
    //         {
    //             tracing::info!("正在终止旧的MT5-{}进程，PID: {}", self.terminal_id, pid);
    //             // 仅终止特定PID的进程，不再通过进程名终止
    //             // 完整命令: taskkill /F /PID <pid>
    //             let _ = StdCommand::new("taskkill")
    //                 .args(&["/F", "/PID", &pid.to_string()])
    //                 .output();
    //         }
    //     }

    //     // 先释放锁，避免长时间持有
    //     drop(mt5_process);

    //     // 如果有旧进程，等待其完全终止
    //     if let Some(pid) = old_pid {
    //         // 等待进程终止
    //         let max_wait = 5; // 最多等待5秒
    //         let mut wait_count = 0;

    //         loop {
    //             #[cfg(windows)]
    //             {
    //                 // 检查进程是否仍在运行
    //                 // 完整命令: tasklist /FI "PID eq <pid>" /FO CSV
    //                 let output = StdCommand::new("tasklist")
    //                     .args(&["/FI", &format!("PID eq {}", pid), "/FO", "CSV"])
    //                     .output()
    //                     .unwrap_or_else(|e| {
    //                         tracing::warn!("检查进程状态失败: {}", e);
    //                         std::process::Output {
    //                             status: std::process::ExitStatus::from_raw(0),
    //                             stdout: Vec::new(),
    //                             stderr: Vec::new(),
    //                         }
    //                     });

    //                 let output_str = String::from_utf8_lossy(&output.stdout);
    //                 if !output_str.contains(&pid.to_string()) {
    //                     // 进程已终止
    //                     tracing::info!("旧的MT5-{}进程(PID:{})已成功终止", self.terminal_id, pid);
    //                     break;
    //                 }
    //             }

    //             wait_count += 1;
    //             if wait_count >= max_wait {
    //                 tracing::warn!("等待旧进程终止超时，将继续操作");
    //                 break;
    //             }

    //             // 等待1秒
    //             tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    //         }
    //     }

    //     // 清理可能存在的同名进程（通过进程名进行精确匹配）
    //     #[cfg(windows)]
    //     {
    //         tracing::info!(process_name = %self.process_name, "check if there is a process with the same name");

    //         // 使用tasklist命令查找特定名称的进程
    //         // 完整命令: tasklist /FI "IMAGENAME eq <process_name>" /FO CSV
    //         let output = StdCommand::new("tasklist")
    //             .args(&["/FI", &format!("IMAGENAME eq {}", self.process_name), "/FO", "CSV"])
    //             .output()
    //             .unwrap_or_else(|e| {
    //                 tracing::warn!("检查进程状态失败: {}", e);
    //                 std::process::Output {
    //                     status: std::process::ExitStatus::from_raw(0),
    //                     stdout: Vec::new(),
    //                     stderr: Vec::new(),
    //                 }
    //             });

    //         let output_str = String::from_utf8_lossy(&output.stdout);
    //         if output_str.contains(&self.process_name) {
    //             tracing::warn!(process_name = %self.process_name, "found a process with the same name, cleaning...");

    //             // 使用进程名精确匹配终止进程，不使用通配符
    //             // 完整命令: taskkill /F /IM <process_name>
    //             let kill_result = StdCommand::new("taskkill")
    //                 .args(&["/F", "/IM", &self.process_name])
    //                 .output();

    //             match kill_result {
    //                 Ok(_) => tracing::info!(process_name = %self.process_name, "cleaned the process"),
    //                 Err(e) => tracing::warn!(process_name = %self.process_name, error = %e, "failed to clean the process"),
    //             }

    //             // 等待进程终止
    //             tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    //         }
    //     }

    //     // 现在创建终端特定的exe副本
    //     let exe_path = create_terminal_exe(self.terminal_id, &self.process_name)?;

    //     // 检查端口是否可用，如果不可用则尝试其他端口
    //     let max_port_tries = 10; // 最多尝试10个端口
    //     let mut port_available = false;

    //     for offset in 0..max_port_tries {
    //         let port = self.server_port + offset;
    //         // 使用socket来检测端口是否被占用
    //         if let Ok(listener) = std::net::TcpListener::bind(format!("127.0.0.1:{}", port)) {
    //             // 如果成功绑定，说明端口可用
    //             drop(listener); // 立即释放端口
    //             port_available = true;
    //             self.server_port = port;
    //             break;
    //         }
    //         tracing::warn!(port = %port, "port is occupied, try next port");
    //     }

    //     if !port_available {
    //         return OtherSnafu {
    //             message: "can't find available port".to_string()
    //         }.fail()?;
    //     }

    //     tracing::info!(terminal_id = %self.terminal_id, port = %self.server_port, "assign port to mt5 backend server");

    //     // 创建子进程，设置新的进程组
    //     let mut command = Command::new(&exe_path);

    //     #[cfg(windows)]
    //     {
    //         command.creation_flags(CREATE_NEW_PROCESS_GROUP.0 as u32);
    //     }

    //     // 添加端口参数
    //     command.arg("--port").arg(self.server_port.to_string());

    //     // 根据debug_output参数决定如何处理输出
    //     let mut child = if debug_output {
    //         // 直接输出到终端
    //         command
    //             .stdout(Stdio::inherit())
    //             .stderr(Stdio::inherit())
    //             .spawn().unwrap();
    //     } else {
    //         // 捕获输出用于日志
    //         command
    //             .stdout(Stdio::piped())
    //             .stderr(Stdio::piped())
    //             .spawn().unwrap();
    //     };

    //     // 保存进程PID用于日志和后续清理
    //     tracing::info!(terminal_id = %self.terminal_id, pid = %child.id().unwrap_or(0), "MT5-{} 进程已启动", self.terminal_id);

    //     // 初始化http客户端
    //     match tokio::time::timeout(tokio::time::Duration::from_secs(30), self.create_mt5_http_client(self.server_port)).await
    //     {
    //         Ok(_) => {
    //             tracing::info!("initialize http client success");
    //         }
    //         Err(e) => {
    //             // 客户端初始化失败，清理进程
    //             if let Err(e) = child.kill().await {
    //                 tracing::error!(terminal_id = %self.terminal_id, error = %e, "failed to kill the process");
    //             }
    //             // 删除临时exe文件
    //             let _ = fs::remove_file(&exe_path);

    //             tracing::error!(terminal_id = %self.terminal_id, error = %e, "failed to initialize http client");
    //             return Err(Mt5Error::internal(format!("failed to initialize http client, error: {}", e)));
    //         }
    //     }

    //     // 检查服务是否启动成功
    //     let is_start_success = self.ping_server().await;
    //     if !is_start_success {
    //         if let Err(e) = child.kill().await {
    //             tracing::error!(terminal_id = %self.terminal_id, error = %e, "failed to kill the process");
    //         }
    //         // 删除临时exe文件
    //         let _ = fs::remove_file(&exe_path);

    //         return Err(Mt5Error::internal(format!("Start MT5-{} server failed, port: {}", self.terminal_id, self.server_port)));
    //     }

    //     // 如果不是直接输出到终端，则捕获输出到日志
    //     if !debug_output {
    //         // 处理标准输出
    //         if let Some(stdout) = child.stdout.take() {
    //             tokio::spawn(async move {
    //                 use tokio::io::{BufReader, AsyncBufReadExt};
    //                 let reader = BufReader::new(stdout);
    //                 let mut lines = reader.lines();

    //                 while let Ok(Some(line)) = lines.next_line().await {
    //                     // tracing::info!("MT5 output: {}", line);
    //                 }
    //             });
    //         }

    //         // 处理标准错误
    //         if let Some(stderr) = child.stderr.take() {
    //             tokio::spawn(async move {
    //                 use tokio::io::{BufReader, AsyncBufReadExt};
    //                 let reader = BufReader::new(stderr);
    //                 let mut lines = reader.lines();

    //                 while let Ok(Some(line)) = lines.next_line().await {
    //                     tracing::warn!(error = %line, "MT5 error");
    //                 }
    //             });
    //         }
    //     }

    //     // 保存进程和exe路径以便后续清理
    //     let mut mt5_process = self.mt5_process.lock().await;
    //     *mt5_process = Some(child);
    //     drop(mt5_process);

    //     let mut exe_path_lock = self.exe_path.lock().await;
    //     *exe_path_lock = Some(exe_path);
    //     drop(exe_path_lock);

    //     tracing::info!(terminal_id = %self.terminal_id, port = %self.server_port, "metatrader5 server started successfully");
    //     Ok(self.server_port)
    // }

    // 直接连接mt5服务器
    pub async fn connect_to_server(&mut self, port: u16) -> Result<(), Mt5Error> {
        self.server_port = port;

        // create http client
        self.create_mt5_http_client(self.server_port).await;

        // check server start success
        let is_ping_success = self.ping_server().await;
        if !is_ping_success {
            return ConnectionSnafu {
                message: "server not start".to_string(),
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }

        Ok(())
    }

    #[instrument(skip_all)]
    async fn ping_server(&mut self) -> bool {
        let mut retry_count = 0;
        let max_retries = 3;
        let mut ping_success = false;
        tracing::info!(terminal_id = %self.terminal_id, "ping mt5 server");
        while retry_count < max_retries {
            let mt5_http_client = self.mt5_http_client.lock().await;
            if let Some(mt5_http_client) = mt5_http_client.as_ref() {
                match mt5_http_client.ping().await {
                    Ok(()) => {
                        ping_success = true;
                        break;
                    }
                    Err(e) => {
                        tracing::error!("{}", e.to_string());
                        retry_count += 1;
                        if retry_count >= max_retries {
                            break;
                        }
                        tracing::warn!(terminal_id = %self.terminal_id, "ping MT5-{} server failed, waiting for retry... ({}/{})", self.terminal_id, retry_count, max_retries);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            } else {
                // HTTP client is None, no point in retrying
                tracing::warn!(terminal_id = %self.terminal_id, "MT5 HTTP client is not initialized, cannot check server status");
                break;
            }
        }
        ping_success
    }

    pub async fn stop_mt5_server(&mut self) -> Result<bool, String> {
        tracing::debug!("开始停止MT5-{}服务", self.terminal_id);
        // 获取并清除进程 - 使用异步锁
        let mut mt5_process = self.mt5_process.lock().await;
        let mut success = false;

        if let Some(mut child) = mt5_process.take() {
            #[cfg(windows)]
            {
                use windows::Win32::System::Console::GenerateConsoleCtrlEvent;

                // 使用进程组 ID 发送信号（进程组 ID 与主进程 ID 相同）
                let pgid = child.id().unwrap_or(0) as u32;
                if pgid != 0 {
                    unsafe {
                        // 第二个参数为进程组 ID
                        if let Err(e) = GenerateConsoleCtrlEvent(0, pgid) {
                            tracing::warn!("发送控制事件到MT5-{}进程失败: {:?}", self.terminal_id, e);
                        }
                    }
                }
            }

            // 释放锁，避免长时间持有
            drop(mt5_process);

            // 增加等待时间，确保子进程有足够时间响应
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // 如果还有进程在运行，强制结束
            if let Ok(None) = child.try_wait() {
                // 终止主进程及其所有子进程
                #[cfg(windows)]
                {
                    // 仅通过PID结束进程
                    if let Some(pid) = child.id() {
                        // 完整命令: taskkill /F /T /PID <pid>
                        match StdCommand::new("taskkill")
                            .args(&["/F", "/T", "/PID", &pid.to_string()])
                            .output()
                        {
                            Ok(_) => tracing::info!("强制终止MT5-{}进程成功，PID: {}", self.terminal_id, pid),
                            Err(e) => {
                                tracing::warn!("强制终止MT5-{}进程失败，PID: {}, 错误: {}", self.terminal_id, pid, e)
                            }
                        }
                    }
                }

                #[cfg(not(windows))]
                {
                    if let Err(e) = child.kill().await {
                        tracing::error!("停止MT5-{}服务失败: {}", self.terminal_id, e);
                        return Ok(false);
                    }
                }
            }

            // 二次检查进程是否已完全停止
            #[cfg(windows)]
            {
                // 最多尝试3次检查
                for attempt in 1..=3 {
                    // 使用tasklist命令查找特定名称的进程
                    let output = StdCommand::new("tasklist")
                        .args(&["/FI", &format!("IMAGENAME eq {}", self.process_name), "/FO", "CSV"])
                        .output()
                        .unwrap_or_else(|e| {
                            tracing::warn!("检查进程状态失败: {}", e);
                            std::process::Output {
                                status: std::process::ExitStatus::from_raw(0),
                                stdout: Vec::new(),
                                stderr: Vec::new(),
                            }
                        });

                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if !output_str.contains(&self.process_name) {
                        // 进程已经完全停止
                        tracing::info!("MT5-{} 服务已完全停止", self.terminal_id);
                        success = true;
                        break;
                    } else {
                        // 如果仍然存在进程，再次尝试终止
                        if attempt < 3 {
                            tracing::warn!(
                                "MT5-{}进程仍在运行，尝试再次终止 (尝试 {}/3)",
                                self.terminal_id,
                                attempt
                            );
                            // 完整命令: taskkill /F /IM <process_name>
                            let _ = StdCommand::new("taskkill")
                                .args(&["/F", "/IM", &self.process_name])
                                .output();
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        } else {
                            tracing::error!("无法停止MT5-{}进程，多次尝试后仍在运行", self.terminal_id);
                            return Ok(false);
                        }
                    }
                }
            }

            #[cfg(not(windows))]
            {
                // 在非Windows系统上，我们假设杀死进程后它已经停止
                success = true;
            }

            tracing::info!(
                "MT5-{} 服务已{}停止",
                self.terminal_id,
                if success { "成功" } else { "尝试" }
            );
        } else {
            // 如果没有进程，释放锁
            drop(mt5_process);

            // 仍然检查是否有同名进程在运行
            #[cfg(windows)]
            {
                let output = StdCommand::new("tasklist")
                    .args(&["/FI", &format!("IMAGENAME eq {}", self.process_name), "/FO", "CSV"])
                    .output()
                    .unwrap_or_else(|e| {
                        tracing::warn!("检查进程状态失败: {}", e);
                        std::process::Output {
                            status: std::process::ExitStatus::from_raw(0),
                            stdout: Vec::new(),
                            stderr: Vec::new(),
                        }
                    });

                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains(&self.process_name) {
                    // 发现同名进程，尝试终止
                    tracing::warn!("发现同名的MT5-{}进程，尝试终止", self.terminal_id);
                    // 完整命令: taskkill /F /IM <process_name>
                    match StdCommand::new("taskkill")
                        .args(&["/F", "/IM", &self.process_name])
                        .output()
                    {
                        Ok(_) => {
                            // 等待进程终止
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                            // 再次检查
                            // 完整命令: tasklist /FI "IMAGENAME eq <process_name>" /FO CSV
                            let check_output = StdCommand::new("tasklist")
                                .args(&["/FI", &format!("IMAGENAME eq {}", self.process_name), "/FO", "CSV"])
                                .output()
                                .unwrap_or_else(|e| {
                                    tracing::warn!("检查进程状态失败: {}", e);
                                    std::process::Output {
                                        status: std::process::ExitStatus::from_raw(0),
                                        stdout: Vec::new(),
                                        stderr: Vec::new(),
                                    }
                                });

                            let check_output_str = String::from_utf8_lossy(&check_output.stdout);
                            success = !check_output_str.contains(&self.process_name);
                            tracing::info!(
                                "同名的MT5-{}进程已{}终止",
                                self.terminal_id,
                                if success { "成功" } else { "尝试但未能" }
                            );
                        }
                        Err(e) => {
                            tracing::warn!("终止同名的MT5-{}进程失败: {}", self.terminal_id, e);
                            success = false;
                        }
                    }
                } else {
                    // 没有找到同名进程，返回成功
                    tracing::info!("未发现MT5-{}进程运行，无需停止", self.terminal_id);
                    success = true;
                }
            }

            #[cfg(not(windows))]
            {
                // 在非Windows系统上，我们假设没有相关进程
                success = true;
            }
        }

        Ok(success)
    }

    async fn create_mt5_http_client(&mut self, port: u16) {
        let mt5_http_client = Mt5HttpClient::new(self.terminal_id, port);
        self.mt5_http_client.lock().await.replace(mt5_http_client);
    }

    // pub async fn ping(&mut self) -> Result<(), ExchangeClientError> {
    //     let mt5_http_client = self.mt5_http_client.lock().await;
    //     if let Some(mt5_http_client) = mt5_http_client.as_ref() {
    //         mt5_http_client.ping().await
    //     } else {
    //         Err(ExchangeClientError::from("MT5 HTTP客户端未初始化"))
    //     }
    // }

    #[instrument(skip_all)]
    pub async fn initialize_terminal(&mut self) -> Result<(), Mt5Error> {
        tracing::info!(terminal_id = %self.terminal_id, "start to initialize terminal");
        let mt5_http_client = self.mt5_http_client.lock().await;

        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            tracing::debug!(terminal_id = %self.terminal_id, "http client is initialized, ready to initialize terminal");
            mt5_http_client
                .initialize_terminal(self.login, &self.password, &self.server, &self.terminal_path)
                .await?;

            tracing::info!(terminal_id = %self.terminal_id, "terminal is initializing, waiting for connection ready");

            let max_retries = 10;
            let mut retry_count = 0;
            while retry_count < max_retries {
                let result = mt5_http_client.get_terminal_info().await;
                if result.is_ok() {
                    tracing::info!(terminal_id = %self.terminal_id, "terminal is initialized successfully");
                    return Ok(());
                }
                retry_count += 1;
                tracing::debug!(terminal_id = %self.terminal_id, "get terminal info failed, retry... ({}/{})", retry_count, max_retries);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            return GetTerminalInfoSnafu {
                message: "the terminal is initialized, but cannot get terminal info".to_string(),
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        } else {
            return TerminalNotInitializedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }
}

#[async_trait]
impl ExchangeClient for MetaTrader5 {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ExchangeClient> {
        Box::new(self.clone())
    }

    fn exchange_type(&self) -> Exchange {
        Exchange::Metatrader5(self.server.clone())
    }

    async fn get_symbol_list(&self) -> Result<Vec<Symbol>, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let symbols = mt5_http_client.get_symbol_list().await?;
            let data_processor = self.data_processor.lock().await;
            let symbols = data_processor.process_symbol_list(symbols).await?;
            Ok(symbols)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    fn get_support_kline_intervals(&self) -> Vec<KlineInterval> {
        Mt5KlineInterval::to_list()
            .iter()
            .map(|interval| KlineInterval::from(interval.clone()))
            .collect()
    }

    async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, ExchangeClientError> {
        Ok(serde_json::Value::Null)
    }

    async fn get_kline_series(
        &self,
        symbol: &str,
        interval: KlineInterval,
        limit: u32,
    ) -> Result<Vec<Kline>, ExchangeClientError> {
        let mt5_interval = Mt5KlineInterval::from(interval);
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let kline_series = mt5_http_client
                .get_kline_series(symbol, mt5_interval.clone(), limit)
                .await?;
            let data_processor = self.data_processor.lock().await;
            let kline_series = data_processor
                .process_kline_series(symbol, mt5_interval, kline_series)
                .await?;
            Ok(kline_series)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn connect_websocket(&mut self) -> Result<(), ExchangeClientError> {
        let (websocket_state, _) = Mt5WsClient::connect_default(self.server_port)
            .await
            .context(WebSocketSnafu {
                message: "connect to metatrader5 websocket server failed".to_string(),
                account_id: self.terminal_id,
                url: format!("ws://localhost:{}/ws", self.server_port),
            })?;
        self.websocket_state.lock().await.replace(websocket_state);
        Ok(())
    }

    async fn subscribe_kline_stream(
        &self,
        symbol: &str,
        interval: KlineInterval,
        frequency: u32,
    ) -> Result<(), ExchangeClientError> {
        let mt5_interval = Mt5KlineInterval::from(interval).to_string();
        let mut mt5_ws_client = self.websocket_state.lock().await;
        tracing::debug!(
            "Metatrader5订阅k线流: {:?}, {:?}, {:?}",
            symbol,
            mt5_interval,
            frequency
        );
        if let Some(state) = mt5_ws_client.as_mut() {
            let params = json!({
                "symbol": symbol,
                "interval": mt5_interval,
            });
            tracing::debug!("Metatrader5订阅k线流参数: {:?}", params);

            state
                .subscribe(Some("kline"), Some(params), Some(frequency))
                .await
                .expect("订阅k线流失败");
        }
        Ok(())
    }

    async fn unsubscribe_kline_stream(
        &self,
        symbol: &str,
        interval: KlineInterval,
        frequency: u32,
    ) -> Result<(), ExchangeClientError> {
        tracing::info!("取消订阅k线流: {:?}", symbol);
        let mt5_interval = Mt5KlineInterval::from(interval).to_string();
        let mut mt5_ws_client = self.websocket_state.lock().await;
        if let Some(state) = mt5_ws_client.as_mut() {
            let params = json!({
                "symbol": symbol,
                "interval": mt5_interval,
            });

            state
                .unsubscribe(Some("kline"), Some(params), Some(frequency))
                .await
                .expect("取消订阅k线流失败");
        }
        Ok(())
    }

    async fn get_socket_stream(&self) -> Result<(), ExchangeClientError> {
        // 判断当前是否正在处理流
        if self.is_process_stream.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::warn!("metatrader5已开始处理流数据, 无需重复获取!");
            return Ok(());
        }
        tracing::debug!("metatrader5开始处理流数据");
        // 如果当前没有处理流，则开始处理流,设置状态为true
        self.is_process_stream.store(true, std::sync::atomic::Ordering::Relaxed);

        let websocket_state = self.websocket_state.clone();
        let data_processor = self.data_processor.clone();

        let future = async move {
            loop {
                let receive_message = {
                    let mut websocket_state = websocket_state.lock().await;
                    if let Some(state) = websocket_state.as_mut() {
                        state.as_mut().next().await
                    } else {
                        None
                    }
                }; // 锁在这里被释放

                // 处理原始数据
                if let Some(Ok(msg)) = receive_message {
                    match msg {
                        Message::Ping(data) => {
                            // tracing::debug!("收到ping帧");
                            let mut websocket_state = websocket_state.lock().await;
                            if let Some(state) = websocket_state.as_mut() {
                                // 回复pong帧
                                let socket = state.as_mut();
                                socket.send(Message::Pong(data)).await.expect("发送pong帧失败");
                                // tracing::debug!("发送pong帧");
                            }
                        }
                        Message::Pong(_) => {
                            tracing::debug!("收到pong帧");
                        }
                        Message::Text(text) => {
                            let stream_json = serde_json::from_str::<serde_json::Value>(&text.to_string())
                                .expect("解析WebSocket消息JSON失败");
                            // tracing::debug!("收到消息: {:?}", stream_json);
                            let data_processor = data_processor.lock().await;
                            if let Err(e) = data_processor.process_stream(stream_json).await {
                                tracing::error!("Failed to process stream data: {}", e);
                                // Consider reconnection logic
                            }
                        }
                        _ => {
                            tracing::debug!("收到其他类型的消息: {:?}", msg);
                        }
                    }
                }
            }
        };
        tokio::spawn(future);
        Ok(())
    }

    // 获取k线历史
    async fn get_kline_history(
        &self,
        symbol: &str,
        interval: KlineInterval,
        time_range: TimeRange,
    ) -> Result<Vec<Kline>, ExchangeClientError> {
        let mt5_interval = Mt5KlineInterval::from(interval);
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let kline_history = mt5_http_client
                .get_kline_history(symbol, mt5_interval.clone(), time_range)
                .await?;
            let data_processor = self.data_processor.lock().await;
            let klines = data_processor
                .process_kline_series(symbol, mt5_interval, kline_history)
                .await?;
            Ok(klines)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn create_order(&self, params: CreateOrderParams) -> Result<Box<dyn OriginalOrder>, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        let mt5_order_request = Mt5CreateOrderParams::from(params);

        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            // 创建订单
            let create_order_result = mt5_http_client.create_order(mt5_order_request).await?;

            // 获取返回码
            let retcode = create_order_result["data"]["retcode"].as_i64().context(RetcodeSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            })?;

            if retcode != 10009 {
                return RetcodeSnafu {
                    terminal_id: self.terminal_id,
                    port: self.server_port,
                }
                .fail()?;
            }

            // 获取订单ID
            let order_id = create_order_result["data"]["order_id"].as_i64().context(OrderIdSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            })?;

            // 获取订单详情
            let order_info = mt5_http_client.get_order(&order_id).await?;

            // 处理订单数据
            let data_processor = self.data_processor.lock().await;
            let order = data_processor.process_order(order_info).await?;
            Ok(order)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn update_order(&self, order: Order) -> Result<Order, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let order_info = mt5_http_client.get_order(&order.exchange_order_id).await?;

            let data_processor = self.data_processor.lock().await;
            let updated_order = data_processor.update_order(order_info, order).await?;
            Ok(updated_order)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn get_transaction_detail(
        &self,
        params: GetTransactionDetailParams,
    ) -> Result<Box<dyn OriginalTransaction>, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let data_processor = self.data_processor.lock().await;

            if let Some(transaction_id) = params.transaction_id {
                let transaction_detail_info = mt5_http_client.get_deal_by_deal_id(&transaction_id).await?;
                let transaction_detail = data_processor.process_deal(transaction_detail_info).await?;
                return Ok(transaction_detail);
            } else if let Some(position_id) = params.position_id {
                let transaction_detail_info = mt5_http_client.get_deal_by_position_id(&position_id).await?;
                let transaction_detail = data_processor.process_deal(transaction_detail_info).await?;
                return Ok(transaction_detail);
            } else if let Some(order_id) = params.order_id {
                let transaction_detail_info = mt5_http_client.get_deals_by_order_id(&order_id).await?;
                let transaction_detail = data_processor.process_deal(transaction_detail_info).await?;
                return Ok(transaction_detail);
            } else {
                return OtherSnafu {
                    message: "transaction_id, position_id, order_id cannot be None".to_string(),
                }
                .fail()?;
            }
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn get_position(&self, params: GetPositionParam) -> Result<Box<dyn OriginalPosition>, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let position_info = mt5_http_client.get_position(&params.position_id).await?;
            let position_list = position_info["data"].clone();
            // 如果仓位列表为空，则说明仓位已平仓
            if position_list.as_array().expect("转换为array失败").len() == 0 {
                return OtherSnafu {
                    message: "仓位已平仓".to_string(),
                }
                .fail()?;
            }
            let data_processor = self.data_processor.lock().await;
            let position = data_processor.process_position(position_list[0].clone()).await?;
            Ok(position)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn get_latest_position(&self, position: &Position) -> Result<Position, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let original_position_json = mt5_http_client
                .get_position(&position.exchange_position_id)
                .await
                .expect("更新仓位失败");
            let position_list = original_position_json["data"].clone();
            // 如果仓位列表为空，则说明仓位已平仓
            if position_list.as_array().expect("转换为array失败").len() == 0 {
                return OtherSnafu {
                    message: "仓位已平仓".to_string(),
                }
                .fail()?;
            }
            let data_processor = self.data_processor.lock().await;
            let position = data_processor
                .process_latest_position(position_list[0].clone(), position)
                .await
                .expect("处理仓位失败");
            Ok(position)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn get_position_number(
        &self,
        position_number_request: GetPositionNumberParams,
    ) -> Result<PositionNumber, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let mt5_position_number_request = Mt5GetPositionNumberParams::from(position_number_request);
            let position_number_info = mt5_http_client
                .get_position_number(mt5_position_number_request)
                .await
                .expect("获取仓位数量失败");
            let mt5_data_processor = self.data_processor.lock().await;
            let position_number = mt5_data_processor
                .process_position_number(position_number_info)
                .await
                .expect("解析position_number数据失败");
            Ok(position_number)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }

    async fn get_account_info(&self) -> Result<Box<dyn OriginalAccountInfo>, ExchangeClientError> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let account_info = mt5_http_client.get_account_info().await?;
            let data_processor = self.data_processor.lock().await;
            let account_info = data_processor
                .process_account_info(self.terminal_id, account_info)
                .await?;
            Ok(account_info)
        } else {
            return HttpClientNotCreatedSnafu {
                terminal_id: self.terminal_id,
                port: self.server_port,
            }
            .fail()?;
        }
    }
}
