mod mt5_http_client;
mod mt5_ws_client;
mod url;
mod mt5_data_processor;

use rust_embed::Embed;
use tempfile::TempDir;
use std::fs;
use tokio::process::Command;
use mt5_http_client::Mt5HttpClient;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex;
use tokio::process::Child;
use windows::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;
use strum::{EnumString, Display};
use serde::{Serialize, Deserialize};
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

#[derive(Embed)]
#[folder = "src/metatrader5/bin/windows/"]
struct Asset;


#[derive(Clone, Display, Serialize, Deserialize, Debug, EnumString, Eq, PartialEq, Hash)]
pub enum Mt5KlineInterval {
    #[strum(serialize = "M1")]
    Minutes1,
    #[strum(serialize = "M5")]
    Minutes5,
    #[strum(serialize = "M15")]
    Minutes15,
    #[strum(serialize = "M30")]
    Minutes30,
    #[strum(serialize = "H1")]
    Hours1,
    #[strum(serialize = "H2")]
    Hours2,
    #[strum(serialize = "H4")]
    Hours4,
    #[strum(serialize = "H6")]
    Hours6,
    #[strum(serialize = "H8")]
    Hours8,
    #[strum(serialize = "H12")]
    Hours12,
    #[strum(serialize = "D1")]
    Days1,
    #[strum(serialize = "W1")]
    Weeks1,
    #[strum(serialize = "MN1")]
    Months1,
}

// 将KlineInterval转换为BinanceKlineInterval
impl From<KlineInterval> for Mt5KlineInterval {
    fn from(interval: KlineInterval) -> Self {
        match interval {
            KlineInterval::Minutes1 => Mt5KlineInterval::Minutes1,
            KlineInterval::Minutes5 => Mt5KlineInterval::Minutes5,
            KlineInterval::Minutes15 => Mt5KlineInterval::Minutes15,
            KlineInterval::Minutes30 => Mt5KlineInterval::Minutes30,
            KlineInterval::Hours1 => Mt5KlineInterval::Hours1,
            KlineInterval::Hours2 => Mt5KlineInterval::Hours2,
            KlineInterval::Hours4 => Mt5KlineInterval::Hours4,
            KlineInterval::Hours6 => Mt5KlineInterval::Hours6,
            KlineInterval::Hours8 => Mt5KlineInterval::Hours8,
            KlineInterval::Hours12 => Mt5KlineInterval::Hours12,
            KlineInterval::Days1 => Mt5KlineInterval::Days1,
            KlineInterval::Weeks1 => Mt5KlineInterval::Weeks1,
            KlineInterval::Months1 => Mt5KlineInterval::Months1,

        }
    }
}

// 将BinanceKlineInterval转换为KlineInterval
impl Into<KlineInterval> for Mt5KlineInterval {
    fn into(self) -> KlineInterval {
        match self {
            Mt5KlineInterval::Minutes1 => KlineInterval::Minutes1,
            Mt5KlineInterval::Minutes5 => KlineInterval::Minutes5,
            Mt5KlineInterval::Minutes15 => KlineInterval::Minutes15,
            Mt5KlineInterval::Minutes30 => KlineInterval::Minutes30,
            Mt5KlineInterval::Hours1 => KlineInterval::Hours1,
            Mt5KlineInterval::Hours2 => KlineInterval::Hours2,
            Mt5KlineInterval::Hours4 => KlineInterval::Hours4,
            Mt5KlineInterval::Hours6 => KlineInterval::Hours6,
            Mt5KlineInterval::Hours8 => KlineInterval::Hours8,
            Mt5KlineInterval::Hours12 => KlineInterval::Hours12,
            Mt5KlineInterval::Days1 => KlineInterval::Days1,
            Mt5KlineInterval::Weeks1 => KlineInterval::Weeks1,
            Mt5KlineInterval::Months1 => KlineInterval::Months1,
        }
    }
}


pub struct MetaTrader5AccountConfig {
    pub account_id: String,
    pub password: String,
    pub server: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct SubscribedSymbol {
    pub symbol: String,
    pub kline_interval: Mt5KlineInterval,
}


pub struct MetaTrader5 {
    mt5_http_client: Arc<Mutex<Mt5HttpClient>>,
    mt5_process: Arc<StdMutex<Option<Child>>>,
    subscribed_symbols: Arc<Mutex<Vec<SubscribedSymbol>>>, // 已订阅的symbol
    websocket_state: Arc<Mutex<Option<WebSocketState>>>,
    data_processor: Arc<Mutex<Mt5DataProcessor>>,
    is_process_stream: Arc<AtomicBool>,
    event_publisher: Arc<Mutex<EventPublisher>>,
}


impl MetaTrader5 {
    pub fn new(event_publisher: EventPublisher) -> Self {
        let event_publisher = Arc::new(Mutex::new(event_publisher));
        Self {
            mt5_http_client: Arc::new(Mutex::new(Mt5HttpClient::new())),
            mt5_process: Arc::new(StdMutex::new(None)),
            subscribed_symbols: Arc::new(Mutex::new(vec![])), // 已订阅的symbol
            websocket_state: Arc::new(Mutex::new(None)),
            is_process_stream: Arc::new(AtomicBool::new(false)),
            event_publisher: event_publisher.clone(),
            data_processor: Arc::new(Mutex::new(Mt5DataProcessor::new(event_publisher))),

        }
    }

    pub async fn start_mt5_server(&self, debug_output: bool) -> Result<(), Box<dyn std::error::Error>> {
        let py_exe = Asset::get("MetaTrader5-x86_64-pc-windows-msvc.exe")
            .ok_or("获取python可执行文件失败")?;
        
        let temp_dir = TempDir::new()?;
        let exe_path = temp_dir.path().join("MetaTrader5.exe");
        fs::write(&exe_path, py_exe.data)?;

        // 创建子进程，设置新的进程组
        let mut command = Command::new(exe_path);
        
        #[cfg(windows)]
        {
            command.creation_flags(CREATE_NEW_PROCESS_GROUP.0 as u32);
        }

        // 添加-u参数禁用Python输出缓冲
        command.arg("-u");
        
        // 根据debug_output参数决定如何处理输出
        let mut child = if debug_output {
            // 直接输出到终端
            command
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?
        } else {
            // 捕获输出用于日志
            command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
        };

        // 如果不是直接输出到终端，则捕获输出到日志
        if !debug_output {
            // 处理标准输出
            if let Some(stdout) = child.stdout.take() {
                tokio::spawn(async move {
                    use tokio::io::{BufReader, AsyncBufReadExt};
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    
                    while let Ok(Some(line)) = lines.next_line().await {
                        tracing::info!("MT5 output: {}", line);
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
        Ok(())
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

    pub async fn connect_websocket(&mut self) -> Result<(), String> {
        let (websocket_state, _) = Mt5WsClient::connect_default().await.unwrap();
        self.websocket_state = Arc::new(Mutex::new(Some(websocket_state)));
        Ok(())
    }

    pub async fn subscribe_kline_stream(&mut self, symbol: &str, kline_interval: KlineInterval, frequency: u32) -> Result<(), String> {
        tracing::info!("订阅k线流: {:?}", symbol);
        let mt5_interval = Mt5KlineInterval::from(kline_interval).to_string();
        let mut mt5_ws_client = self.websocket_state.lock().await;
        if let Some(state) = mt5_ws_client.as_mut() {
            let params = json!({
                "symbol": symbol,
                "interval": mt5_interval,
            });

            state.subscribe(Some("kline"), Some(params), Some(frequency)).await.expect("订阅k线流失败");
        }
        Ok(())
    }

    pub async fn get_socket_stream(&mut self) -> Result<(), String> {
        // 判断当前是否正在处理流
        if self.is_process_stream.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::warn!("metatrader5已开始处理流数据, 无需重复获取!");
            return Ok(());
        }
        tracing::debug!("开始metatrader5处理流数据");
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
                            tracing::debug!("收到ping帧");
                            let mut websocket_state = websocket_state.lock().await;
                            if let Some(state) = websocket_state.as_mut() {
                                // 回复pong帧
                                let socket = state.as_mut();
                                socket.send(Message::Pong(data)).await.expect("发送pong帧失败");
                                tracing::debug!("发送pong帧");
                            }
                        },
                        Message::Pong(_) => {
                            tracing::debug!("收到pong帧");
                        },
                        Message::Text(text) => {
                            let stream_json = serde_json::from_str::<serde_json::Value>(&text.to_string()).expect("解析WebSocket消息JSON失败");
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

    pub async fn ping(&mut self) -> Result<(), String> {
        self.mt5_http_client.lock().await.ping().await
    }

    pub async fn initialize_client(&mut self) -> Result<(), String> {
        self.mt5_http_client.lock().await.initialize_client().await
    }

    pub async fn get_client_status(&mut self) -> Result<(), String> {
        self.mt5_http_client.lock().await.get_client_status().await
    }

    pub async fn login(&mut self, account_id: i32, password: &str, server: &str, terminal_path: &str) -> Result<(), String> {
        self.mt5_http_client.lock().await.login(account_id, password, server, terminal_path).await
    }




    
    
}


impl Drop for MetaTrader5 {
    fn drop(&mut self) {
        // 在对象被销毁时确保进程被关闭
        if let Some(mut child) = self.mt5_process.lock().unwrap().take() {
            // 同步方式结束进程
            let _ = child.start_kill();
            tracing::info!("metatrader5服务已停止");
        }
    }
}

