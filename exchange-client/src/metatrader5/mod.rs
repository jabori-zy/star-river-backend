mod mt5_http_client;
mod mt5_ws_client;
mod url;
mod mt5_data_processor;
mod mt5_types;

use mt5_types::Mt5PositionNumberRequest;
use types::position::PositionNumber;
use mt5_http_client::Mt5HttpClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::market::KlineInterval;
use mt5_ws_client::Mt5WsClient;
use mt5_ws_client::WebSocketState;
use serde_json::json;
use futures::StreamExt;
use std::sync::atomic::AtomicBool;
use tokio_tungstenite::tungstenite::Message;
use futures::SinkExt;
use mt5_data_processor::Mt5DataProcessor;
use event_center::EventPublisher;
use crate::ExchangeClient;
use std::any::Any;
use async_trait::async_trait;
use types::order::{ExchangeOrder, Order};
use event_center::command_event::order_engine_command::CreateOrderParams;
use types::position::{PositionNumberRequest, ExchangePosition, Position};
use super::metatrader5::mt5_types::Mt5CreateOrderParams;
use event_center::command_event::position_engine_command::GetPositionParam;
use super::metatrader5::mt5_types::Mt5KlineInterval;
use event_center::command_event::order_engine_command::GetTransactionDetailParams;
use types::transaction_detail::{TransactionDetail, ExchangeTransactionDetail};
use types::account::ExchangeAccountInfo;
use types::account::mt5_account::Mt5AccountInfo;
use rust_embed::Embed;
use std::fs;
use std::process::Command as StdCommand;
use std::process::Stdio;
use tempfile::TempDir;
use tokio::process::Child;
use tokio::process::Command;
use windows::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;
use std::sync::Mutex as StdMutex;
#[derive(Embed)]
#[folder = "src/metatrader5/bin/windows/"]
struct Asset;



pub struct MetaTrader5AccountConfig {
    pub account_id: String,
    pub password: String,
    pub server: String,
}


#[derive(Clone, Debug)]
pub struct MetaTrader5 {
    pub process_name: String, // 进程名称
    pub server_port: u16, // 服务器端口

    pub terminal_id: i32, // 终端id
    pub login: i64,
    pub password: String,
    pub server: String,
    pub terminal_path: String,
    mt5_http_client: Arc<Mutex<Option<Mt5HttpClient>>>,
    websocket_state: Arc<Mutex<Option<WebSocketState>>>,
    data_processor: Arc<Mutex<Mt5DataProcessor>>,
    is_process_stream: Arc<AtomicBool>,
    event_publisher: Arc<Mutex<EventPublisher>>,
    mt5_process: Arc<StdMutex<Option<Child>>>,
}


impl MetaTrader5 {
    pub fn new(
        terminal_id: i32,
        login: i64,
        password: String,
        server: String,
        terminal_path: String,
        event_publisher: EventPublisher
    ) -> Self {
        let event_publisher = Arc::new(Mutex::new(event_publisher));
        Self {
            process_name: format!("Metatrader5-{}.exe", terminal_id),
            server_port: 8000 + terminal_id as u16,
            terminal_id,
            login,
            password,
            server,
            terminal_path,
            mt5_http_client: Arc::new(Mutex::new(None)),
            websocket_state: Arc::new(Mutex::new(None)),
            is_process_stream: Arc::new(AtomicBool::new(false)),
            event_publisher: event_publisher.clone(),
            data_processor: Arc::new(Mutex::new(Mt5DataProcessor::new(event_publisher))),
            mt5_process: Arc::new(StdMutex::new(None)),
        }
    }

    pub async fn start_mt5_server(&mut self, debug_output: bool) -> Result<u16, String> {
        // 先检查并清理可能存在的旧进程
        #[cfg(windows)]
        {
            // 查找特定名称的进程
            let output = StdCommand::new("tasklist")
                .args(&["/FI", &format!("IMAGENAME eq {}", self.process_name), "/FO", "CSV"])
                .output().map_err(|e| format!("检查并清理可能存在的旧进程失败: {}", e))?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains(&self.process_name) {
                tracing::warn!("发现指定的MetaTrader5进程 {}, 正在清理...", self.process_name);
                
                // 结束特定进程
                let _ = StdCommand::new("taskkill")
                    .args(&["/F", "/IM", &self.process_name])
                    .output().map_err(|e| format!("结束特定进程失败: {}", e))?;
                    
                // 等待进程完全退出
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
        let py_exe = Asset::get("MetaTrader5-x86_64-pc-windows-msvc.exe")
            .ok_or("获取python可执行文件失败")?;
        
        let temp_dir = TempDir::new().map_err(|e| format!("创建临时目录失败: {}", e))?;
        // 将exe文件写入临时目录
        let exe_path = temp_dir.path().join(&self.process_name);
        fs::write(&exe_path, py_exe.data).map_err(|e| format!("写入exe文件失败: {}", e))?;

        // 检查端口是否可用，如果不可用则尝试其他端口
        let max_port_tries = 10; // 最多尝试10个端口
        let mut port_available = false;
        
        for offset in 0..max_port_tries {
            let port = self.server_port + offset;
            // 使用socket来检测端口是否被占用
            if let Ok(listener) = std::net::TcpListener::bind(format!("127.0.0.1:{}", port)) {
                // 如果成功绑定，说明端口可用
                drop(listener); // 立即释放端口
                port_available = true;
                self.server_port = port;
                break;
            }
            tracing::warn!("端口 {} 已被占用，尝试下一个端口", port);
        }
        
        if !port_available {
            return Err(format!("无法找到可用端口，尝试了从 {} 到 {}", self.server_port, self.server_port + max_port_tries - 1).into());
        }
    
        tracing::info!("为 MT5-{} 分配端口: {}", self.terminal_id, self.server_port);

        // 创建子进程，设置新的进程组
        let mut command = Command::new(exe_path);
        
        #[cfg(windows)]
        {
            command.creation_flags(CREATE_NEW_PROCESS_GROUP.0 as u32);
        }

        // 添加-u参数禁用Python输出缓冲
        // command.arg("-u");
        // 仅添加端口参数
        command.arg("--port").arg(self.server_port.to_string());
        
        // 根据debug_output参数决定如何处理输出
        let mut child = if debug_output {
            // 直接输出到终端
            command
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn().map_err(|e| format!("启动进程失败: {}", e))?
        } else {
            // 捕获输出用于日志
            command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn().map_err(|e| format!("启动进程失败: {}", e))?
        };

        // 初始化http客户端
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(30), 
            self.create_mt5_http_client(self.server_port)
        )
        .await
        {
            Ok(_) => {
                tracing::info!("初始化http客户端成功");
            }
            Err(e) => {
                tracing::error!("初始化http客户端失败: {}", e);
                return Err(format!("初始化http客户端失败: {}", e).into());
            }
        }
        // tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        // 检查服务是否启动成功
        let is_start_success = self.check_server_start_success().await.expect("检查服务启动失败");
        if !is_start_success {
            if let Err(e) = child.kill().await {
                tracing::error!("终止失败的MT5-{}进程时出错: {}", self.terminal_id, e);
            }
            return Err(format!("MT5-{} 服务启动失败，无法连接到端口 {}", self.terminal_id, self.server_port).into());
        }

        // 如果不是直接输出到终端，则捕获输出到日志
        if !debug_output {
            // 处理标准输出
            if let Some(stdout) = child.stdout.take() {
                tokio::spawn(async move {
                    use tokio::io::{BufReader, AsyncBufReadExt};
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    
                    while let Ok(Some(line)) = lines.next_line().await {
                        // tracing::info!("MT5 output: {}", line);
                    }
                });
            }

            // 处理标准错误
            if let Some(stderr) = child.stderr.take() {
                tokio::spawn(async move {
                    use tokio::io::{BufReader, AsyncBufReadExt};
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    
                    while let Ok(Some(line)) = lines.next_line().await {
                        tracing::warn!("MT5 error: {}", line);
                    }
                });
            }
        }

        *self.mt5_process.lock().unwrap() = Some(child);

        tracing::info!("metatrader5服务启动成功");
        Ok(self.server_port)
    }

    async fn check_server_start_success(&mut self) -> Result<bool, String> {
        let mut retry_count = 0;
        let max_retries = 10;
        let mut ping_success = false;
        while retry_count < max_retries {
            let mt5_http_client = self.mt5_http_client.lock().await;
            if let Some(mt5_http_client) = mt5_http_client.as_ref() {
                match mt5_http_client.ping().await {
                    Ok(response) if response["message"] == "pong" => {
                        ping_success = true;
                        break;
                    },
                    _ => {
                        retry_count += 1;
                        if retry_count >= max_retries {
                            break;
                        }
                        tracing::warn!("MT5-{} 服务尚未就绪，等待重试... ({}/{})", self.terminal_id, retry_count, max_retries);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }
        Ok(ping_success)
    }


    pub async fn stop_mt5_server(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.mt5_process.lock().unwrap().take() {
            #[cfg(windows)]
            {
                use windows::Win32::System::Console::GenerateConsoleCtrlEvent;
                
                // 使用进程组 ID 发送信号（进程组 ID 与主进程 ID 相同）
                let pgid = child.id().unwrap_or(0) as u32;
                if pgid != 0 {
                    unsafe {
                        // 第二个参数为进程组 ID
                        GenerateConsoleCtrlEvent(0, pgid).expect("发送控制事件失败");
                    }
                }
            }

            // 增加等待时间，确保子进程有足够时间响应
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // 如果还有进程在运行，强制结束
            if let Ok(None) = child.try_wait() {
                // 终止主进程及其所有子进程
                #[cfg(windows)]
                {
                    use std::process::Command;
                    let _ = Command::new("taskkill")
                        .args(&["/F", "/T", "/PID", &child.id().unwrap_or(0).to_string()])
                        .output();
                    // 同时尝试通过进程名称结束特定进程
                    let process_name = format!("Metatrader5-{}.exe", self.terminal_id);
                    let _ = StdCommand::new("taskkill")
                        .args(&["/F", "/IM", &process_name])
                        .output();
                }
                
                #[cfg(not(windows))]
                {
                    child.kill().await.expect("停止MT5服务失败");
                }
            }
            
            tracing::info!("metatrader5服务已停止");
        }
        Ok(())
    }

    async fn create_mt5_http_client(&mut self, port: u16) -> Result<(), String> {
        let mt5_http_client = Mt5HttpClient::new(port);
        self.mt5_http_client.lock().await.replace(mt5_http_client);
        Ok(())
    }


    // pub async fn connect_websocket(&mut self) -> Result<(), String> {
    //     let (websocket_state, _) = Mt5WsClient::connect_default().await.unwrap();
    //     self.websocket_state.lock().await.replace(websocket_state);
    //     Ok(())
    // }

    // pub async fn subscribe_kline_stream(&mut self, symbol: &str, kline_interval: KlineInterval, frequency: u32) -> Result<(), String> {
    //     let mt5_interval = Mt5KlineInterval::from(kline_interval).to_string();
    //     let mut mt5_ws_client = self.websocket_state.lock().await;
    //     tracing::debug!("Metatrader5订阅k线流: {:?}, {:?}, {:?}", symbol, mt5_interval, frequency);
    //     if let Some(state) = mt5_ws_client.as_mut() {
    //         let params = json!({
    //             "symbol": symbol,
    //             "interval": mt5_interval,
    //         });
    //         tracing::debug!("Metatrader5订阅k线流参数: {:?}", params);

    //         state.subscribe(Some("kline"), Some(params), Some(frequency)).await.expect("订阅k线流失败");
    //     }
    //     Ok(())
    // }

    pub async fn ping(&mut self) -> Result<serde_json::Value, String> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            mt5_http_client.ping().await
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }

    pub async fn initialize_terminal(&mut self) -> Result<(), String> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            mt5_http_client.initialize_terminal(self.login, &self.password, &self.server, &self.terminal_path).await?;
            
            tracing::info!("MT5-{} 终端初始化中，等待连接就绪...", self.terminal_id);
            
            let max_retries = 10;
            let mut retry_count = 0;
            while retry_count < max_retries {
                let result = mt5_http_client.get_terminal_info().await;
                tracing::info!("MT5-{} 终端初始化结果: {:?}", self.terminal_id, result);
                if result.is_ok() {
                    tracing::info!("MT5-{} 终端连接成功", self.terminal_id);
                    return Ok(());
                }
                retry_count += 1;
                tracing::debug!("MT5-{} 终端尚未就绪，等待重试... ({}/{})", self.terminal_id, retry_count, max_retries);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            
            // 如果多次重试后仍然失败，则返回错误
            Err(format!("MT5-{} 终端初始化超时，请检查终端是否正常启动", self.terminal_id))
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }


    pub async fn login(&self) -> Result<serde_json::Value, String> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            mt5_http_client.login(self.login, &self.password, &self.server).await
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }

}


#[async_trait]
impl ExchangeClient for MetaTrader5 {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn ExchangeClient> {
        Box::new(self.clone())
    }

    async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, String> {
        Ok(serde_json::Value::Null)
    }

    async fn get_kline_series(&self, symbol: &str, interval: KlineInterval, limit: Option<u32>) -> Result<(), String> {
        let mt5_interval = Mt5KlineInterval::from(interval);
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let kline_series = mt5_http_client.get_kline_series(symbol, mt5_interval.clone(), limit).await.expect("获取k线系列失败");
            tracing::info!("获取k线系列成功, k线数量: {:?}", kline_series);
            let data_processor = self.data_processor.lock().await;
            data_processor.process_kline_series(symbol, mt5_interval, kline_series).await;
            Ok(())
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }

    async fn connect_websocket(&mut self) -> Result<(), String> {
        tracing::debug!("Metatrader5连接websocket");
        let (websocket_state, _) = Mt5WsClient::connect_default(self.server_port).await.unwrap();
        self.websocket_state.lock().await.replace(websocket_state);
        Ok(())
    }

    async fn subscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), String> {
        let mt5_interval = Mt5KlineInterval::from(interval).to_string();
        let mut mt5_ws_client = self.websocket_state.lock().await;
        tracing::debug!("Metatrader5订阅k线流: {:?}, {:?}, {:?}", symbol, mt5_interval, frequency);
        if let Some(state) = mt5_ws_client.as_mut() {
            let params = json!({
                "symbol": symbol,
                "interval": mt5_interval,
            });
            tracing::debug!("Metatrader5订阅k线流参数: {:?}", params);

            state.subscribe(Some("kline"), Some(params), Some(frequency)).await.expect("订阅k线流失败");
        }
        Ok(())
    }

    async fn unsubscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), String> {
        tracing::info!("取消订阅k线流: {:?}", symbol);
        let mt5_interval = Mt5KlineInterval::from(interval).to_string();
        let mut mt5_ws_client = self.websocket_state.lock().await;
        if let Some(state) = mt5_ws_client.as_mut() {
            let params = json!({
                "symbol": symbol,
                "interval": mt5_interval,
            });

            state.unsubscribe(Some("kline"), Some(params), Some(frequency)).await.expect("取消订阅k线流失败");
        }
        Ok(())
    }

    async fn get_socket_stream(&self) -> Result<(), String> {
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
                };  // 锁在这里被释放
                
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
                        },
                        Message::Pong(_) => {
                            tracing::debug!("收到pong帧");
                        },
                        Message::Text(text) => {
                            let stream_json = serde_json::from_str::<serde_json::Value>(&text.to_string()).expect("解析WebSocket消息JSON失败");
                            tracing::debug!("收到消息: {:?}", stream_json);
                            let data_processor = data_processor.lock().await;
                            data_processor.process_stream(stream_json).await;
  
                        },
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

    async fn create_order(&self, params: CreateOrderParams) -> Result<Box<dyn ExchangeOrder>, String> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        let mt5_order_request = Mt5CreateOrderParams::from(params);
        // 创建订单
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let create_order_result = mt5_http_client.create_order(mt5_order_request).await.expect("创建订单失败");
            // 根据创建的订单，获取订单的信息
            let retcode = create_order_result["data"]["retcode"].as_i64().expect("获取retcode失败");
            if retcode != 10009 {
                return Err(format!("创建订单失败, retcode: {}", retcode));
            }
            let order_id = create_order_result["data"]["order_id"].as_i64().expect("获取order_id失败");
            let order_info = mt5_http_client.get_order(&order_id).await.expect("获取订单失败");

            let data_processor = self.data_processor.lock().await;
            let order = data_processor.process_order(order_info).await.expect("处理订单失败");

            // 入库
            
            Ok(order)
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }

    async fn update_order(&self, order: Order) -> Result<Order, String> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let order_info = mt5_http_client.get_order(&order.exchange_order_id).await.expect("更新订单失败");

            let data_processor = self.data_processor.lock().await;
            let order = data_processor.update_order(order_info, order).await.expect("处理订单失败");
            Ok(order)
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }


    async fn get_transaction_detail(&self, params: GetTransactionDetailParams) -> Result<Box<dyn ExchangeTransactionDetail>, String> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
        // 如果transaction_id不为None，则按照deal_id获取交易明细
            if let Some(transaction_id) = params.transaction_id {
                let transaction_detail_info = mt5_http_client.get_deal_by_deal_id(&transaction_id).await.expect("获取交易明细失败");
                let data_processor = self.data_processor.lock().await;
                let transaction_detail = data_processor.process_deal(transaction_detail_info).await.expect("处理交易明细失败");
                return Ok(transaction_detail);
            } else if let Some(position_id) = params.position_id {
                let transaction_detail_info = mt5_http_client.get_deal_by_position_id(&position_id).await.expect("获取交易明细失败");
                let data_processor = self.data_processor.lock().await;
                let transaction_detail = data_processor.process_deal(transaction_detail_info).await.expect("处理交易明细失败");
                return Ok(transaction_detail);
            } else if let Some(order_id) = params.order_id {
                let transaction_detail_info = mt5_http_client.get_deals_by_order_id(&order_id).await.expect("获取交易明细失败");
                let data_processor = self.data_processor.lock().await;
                let transaction_detail = data_processor.process_deal(transaction_detail_info).await.expect("处理交易明细失败");
                return Ok(transaction_detail);
            } else {
                return Err("交易明细id不能为None".to_string());
            }
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }

    async fn get_position(&self, params: GetPositionParam) -> Result<Box<dyn ExchangePosition>, String> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let position_info = mt5_http_client.get_position(&params.position_id).await.expect("获取仓位失败");
            let data_processor = self.data_processor.lock().await;
            let position = data_processor.process_position(position_info).await.expect("处理仓位失败");
            Ok(position)
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }

    async fn update_position(&self, position: &Position) -> Result<Position, String> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let position_info = mt5_http_client.get_position(&position.exchange_position_id).await.expect("更新仓位失败");
            let data_processor = self.data_processor.lock().await;
            let position = data_processor.update_position(position_info, position).await.expect("处理仓位失败");
            Ok(position)
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }

    async fn get_position_number(&self, position_number_request: PositionNumberRequest) -> Result<PositionNumber, String> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let mt5_position_number_request = Mt5PositionNumberRequest::from(position_number_request);
            let position_number_info = mt5_http_client.get_position_number(mt5_position_number_request).await.expect("获取仓位数量失败");
            let mt5_data_processor = self.data_processor.lock().await;
            let position_number = mt5_data_processor.process_position_number(position_number_info).await.expect("解析position_number数据失败");
            Ok(position_number)
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }

    async fn get_account_info(&self) -> Result<Box<dyn ExchangeAccountInfo>, String> {
        let mt5_http_client = self.mt5_http_client.lock().await;
        if let Some(mt5_http_client) = mt5_http_client.as_ref() {
            let account_info = mt5_http_client.get_account_info().await.expect("获取账户信息失败");
            let data_processor = self.data_processor.lock().await;
            let account_info = data_processor.process_account_info(self.terminal_id, account_info).await.expect("处理账户信息失败");
            Ok(account_info)
        } else {
            Err("MT5 HTTP客户端未初始化".to_string())
        }
    }

}

