use crate::EngineContext;
use crate::EngineName;
use async_trait::async_trait;
use event_center::command_event::exchange_engine_command::RegisterMt5ExchangeParams;
use event_center::command_event::exchange_engine_command::UnregisterMt5ExchangeParams;
use event_center::command_event::exchange_engine_command::{
    ExchangeEngineCommand, RegisterExchangeParams,
};
use event_center::command_event::CommandEvent;
use event_center::response_event::exchange_engine_response::RegisterMt5ExchangeSuccessResponse;
use event_center::response_event::exchange_engine_response::{
    ExchangeEngineResponse, RegisterExchangeSuccessResponse,
};
use event_center::response_event::ResponseEvent;
use event_center::Event;
use event_center::EventPublisher;
use exchange_client::binance::BinanceExchange;
use exchange_client::metatrader5::MetaTrader5;
use exchange_client::ExchangeClient;
use rust_embed::Embed;
use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::process::Command as StdCommand;
use std::process::Stdio;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tempfile::TempDir;
use tokio::process::Child;
use tokio::process::Command;
use tokio::sync::broadcast;
use types::market::Exchange;
use utils::get_utc8_timestamp;
use windows::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;
use sea_orm::DatabaseConnection;
use database::query::mt5_account_config_query::Mt5AccountConfigQuery;

#[derive(Debug)]
pub struct ExchangeEngineContext {
    pub engine_name: EngineName,
    pub exchanges: HashMap<i32, Box<dyn ExchangeClient>>, // 交易所的账户id -> 交易所 每个交易所对应一个账户
    pub event_publisher: EventPublisher,
    pub event_receiver: Vec<broadcast::Receiver<Event>>,
    pub mt5_process: Arc<StdMutex<Option<Child>>>,
    pub is_mt5_server_running: Arc<AtomicBool>, //mt5服务器是否正在运行
    pub database: DatabaseConnection,
}

impl Clone for ExchangeEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            exchanges: self
                .exchanges
                .iter()
                .map(|(id, client)| (id.clone(), client.clone_box()))
                .collect(),
            event_publisher: self.event_publisher.clone(),
            event_receiver: self
                .event_receiver
                .iter()
                .map(|receiver| receiver.resubscribe())
                .collect(),
            mt5_process: self.mt5_process.clone(),
            is_mt5_server_running: self.is_mt5_server_running.clone(),
            database: self.database.clone(),
        }
    }
}

#[async_trait]
impl EngineContext for ExchangeEngineContext {
    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_engine_name(&self) -> EngineName {
        self.engine_name.clone()
    }

    fn get_event_publisher(&self) -> &EventPublisher {
        &self.event_publisher
    }

    fn get_event_receiver(&self) -> Vec<broadcast::Receiver<Event>> {
        self.event_receiver
            .iter()
            .map(|receiver| receiver.resubscribe())
            .collect()
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Command(command_event) => match command_event {
                CommandEvent::ExchangeEngine(exchange_manager_command) => {
                    
                    match exchange_manager_command {
                        ExchangeEngineCommand::RegisterMt5Exchange(register_mt5_exchange_command) => {
                            tracing::debug!("接收到命令: {:?}", register_mt5_exchange_command);
                            self.register_mt5_exchange(register_mt5_exchange_command)
                                .await
                                .expect("注册mt5交易所失败");
                        }
                        ExchangeEngineCommand::UnregisterMt5Exchange(unregister_mt5_exchange_command,) => {
                            tracing::debug!("接收到命令: {:?}", unregister_mt5_exchange_command);
                            self.unregister_mt5_exchange(unregister_mt5_exchange_command)
                                .await
                                .expect("注销mt5交易所失败");
                        }
                        ExchangeEngineCommand::RegisterExchange(register_exchange_command) => {
                            tracing::debug!("接收到命令: {:?}", register_exchange_command);
                            match register_exchange_command.exchange {
                                Exchange::Metatrader5 => {
                                    self.register_mt5_exchange2(register_exchange_command)
                                        .await
                                        .expect("注册mt5交易所失败");
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl ExchangeEngineContext {
    // async fn register_exchange(&mut self, register_params: RegisterExchangeParams) -> Result<(), String>{
    //     // 检查是否已经注册
    //     let should_register = {
    //         !self.exchanges.contains_key(&register_params.exchange)
    //     };

    //     if !should_register {
    //         // 直接发送响应事件
    //         tracing::warn!("{}交易所已注册, 无需重复注册", register_params.exchange);
    //         let response_event = ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterExchangeSuccess(RegisterExchangeSuccessResponse {
    //             exchange: register_params.exchange.clone(),
    //             response_timestamp: get_utc8_timestamp(),
    //             response_id: register_params.request_id,
    //         }));
    //         self.get_event_publisher().publish(response_event.clone().into()).unwrap();
    //         return Ok(());
    //     }

    //     match register_params.exchange {
    //         Exchange::Binance => {
    //             // 当类型为Box<dyn Trait Bound>时，需要显式地指定类型

    //             let mut binance_exchange = Box::new(BinanceExchange::new(self.get_event_publisher().clone())) as Box<dyn ExchangeClient>;
    //             binance_exchange.connect_websocket().await?;

    //             tracing::info!("{}交易所注册成功!", register_params.exchange);
    //             self.exchanges.insert(register_params.exchange.clone(), binance_exchange);
    //             // 发送响应事件

    //             let response_event = ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterExchangeSuccess(RegisterExchangeSuccessResponse {
    //                 exchange: register_params.exchange.clone(),
    //                 response_timestamp: get_utc8_timestamp(),
    //                 response_id: register_params.request_id,
    //             }));
    //             self.get_event_publisher().publish(response_event.clone().into()).unwrap();

    //             Ok(())

    //         }
    //         Exchange::Metatrader5 => {
    //             let mut mt5 = MetaTrader5::new(self.get_event_publisher().clone());
    //             // 启动mt5服务器

    //             mt5.start_mt5_server(false).await.unwrap();
    //             tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    //             mt5.initialize_terminal().await.unwrap();
    //             tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    //             // mt5.login(23643, "HhazJ520!!!!", "EBCFinancialGroupKY-Demo", r"C:\Program Files\MetaTrader 5\terminal64.exe").await.expect("登录失败");
    //             mt5.login(76898751, "HhazJ520....", "Exness-MT5Trial5", r"C:\Program Files\MetaTrader 5\terminal64.exe").await.expect("登录失败");
    //             tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    //             let mut mt5_exchange = Box::new(mt5) as Box<dyn ExchangeClient>;
    //             mt5_exchange.connect_websocket().await?;
    //             tracing::info!("{}交易所注册成功!", register_params.exchange);

    //             self.exchanges.insert(register_params.exchange.clone(), mt5_exchange);
    //             let response_event = ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterExchangeSuccess(RegisterExchangeSuccessResponse {
    //                 exchange: register_params.exchange.clone(),
    //                 response_timestamp: get_utc8_timestamp(),
    //                 response_id: register_params.request_id,
    //             }));
    //             self.get_event_publisher().publish(response_event.clone().into()).unwrap();
    //             Ok(())
    //         }

    //         _ => {
    //             return Err("不支持的交易所".to_string());
    //         }
    //     }
    // }

    pub async fn register_mt5_exchange(&mut self,register_params: RegisterMt5ExchangeParams) -> Result<(), String> {
        // 初始化 mt5 客户端
        let mut mt5 = MetaTrader5::new(
            register_params.account_id,
            register_params.login,
            register_params.password,
            register_params.server,
            register_params.terminal_path,
            self.get_event_publisher().clone(),
        );
        mt5.start_mt5_server(false).await.unwrap();


        // // 启动mt5服务器
        // match tokio::time::timeout(
        //     tokio::time::Duration::from_secs(20), 
        //     mt5.start_mt5_server(false)
        // )
        // .await
        // {
        //     Ok(port) => {
        //         match port {
        //             Ok(port) => {
        //                 tracing::info!("mt5服务器启动成功, 端口: {}", port);
        //             }
        //             Err(e) => {
        //                 tracing::error!("mt5服务器启动失败: {}", e);
        //             }
        //         }
        //     }
        //     Err(_) => {
        //         // 超时
        //         let error_msg = format!("MT5服务启动超时 (20秒)，terminal_id: {}", register_params.account_id);
        //         tracing::error!("{}", error_msg);
        //         return Err(error_msg);
        //     }
        // }

        // 初始化终端
        mt5.initialize_terminal().await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // 注册完成后直接登录
        let login_result = mt5.login().await.unwrap();
        tracing::info!("登录结果: {:?}", login_result);

        // 存储交易所客户端
        let mut mt5_exchange = Box::new(mt5) as Box<dyn ExchangeClient>;
        // 连接websocket
        mt5_exchange.connect_websocket().await?;
        
        tracing::info!("{}交易所注册成功!", Exchange::Metatrader5);
        self.exchanges
            .insert(register_params.account_id, mt5_exchange);

        // 发送响应事件
        let response_event =
            ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterMt5ExchangeSuccess(
                RegisterMt5ExchangeSuccessResponse {
                    terminal_id: register_params.account_id,
                    exchange: Exchange::Metatrader5,
                    response_timestamp: get_utc8_timestamp(),
                    response_id: register_params.request_id,
                },
            ));
        self.get_event_publisher()
            .publish(response_event.clone().into())
            .unwrap();

        Ok(())
    }

    pub async fn register_mt5_exchange2(&mut self,register_params: RegisterExchangeParams) -> Result<(), String> {
        // 从数据库中获取mt5账户配置
        let mt5_account_config = Mt5AccountConfigQuery::get_mt5_account_config_by_id(&self.database, register_params.account_id).await;
        match mt5_account_config {
            Ok(Some(mt5_account_config)) => {
                // 初始化 mt5 客户端
                let mut mt5 = MetaTrader5::new(
                    mt5_account_config.id,
                    mt5_account_config.login,
                    mt5_account_config.password,
                    mt5_account_config.server,
                    mt5_account_config.terminal_path,
                    self.get_event_publisher().clone(),
                );
                // 初始化终端


                // let server_port = mt5.start_mt5_server(false).await.unwrap();

                // 启动mt5服务器
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(30), 
                    mt5.start_mt5_server(false)
                )
                .await
                {
                    Ok(port) => {
                        match port {
                            Ok(port) => {
                                tracing::info!("mt5服务器启动成功, 端口: {}", port);
                            }
                            Err(e) => {
                                tracing::error!("mt5服务器启动失败: {}", e);
                            }
                        }
                    }
                    Err(_) => {
                        // 超时
                        let error_msg = format!("MT5服务启动超时 (20秒)，terminal_id: {}", register_params.account_id);
                        tracing::error!("{}", error_msg);
                        return Err(error_msg);
                    }
                }

                // 初始化终端
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(30), 
                    mt5.initialize_terminal()
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("终端初始化成功");
                    }
                    Err(e) => {
                        tracing::error!("终端初始化失败: {}", e);
                        return Err(e.to_string());
                    }
                }


                // mt5.initialize_terminal().await.unwrap();

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                // // 注册完成后直接登录
                // let login_result = mt5.login().await.unwrap();
                // tracing::info!("登录结果: {:?}", login_result);

                // 连接websocket
                mt5.connect_websocket().await?;

                // 存储交易所客户端
                let mt5_exchange = Box::new(mt5) as Box<dyn ExchangeClient>;

                tracing::info!("{}交易所注册成功!", Exchange::Metatrader5);
                self.exchanges
                    .insert(register_params.account_id, mt5_exchange);

                // 发送响应事件
                let response_event =
                    ResponseEvent::ExchangeEngine(ExchangeEngineResponse::RegisterMt5ExchangeSuccess(
                        RegisterMt5ExchangeSuccessResponse {
                            terminal_id: register_params.account_id,
                            exchange: Exchange::Metatrader5,
                            response_timestamp: get_utc8_timestamp(),
                            response_id: register_params.request_id,
                        },
                    ));
                self.get_event_publisher()
                    .publish(response_event.clone().into())
                    .unwrap();

                Ok(())
            }
            Ok(None) => {
                return Err("账户配置不存在".to_string());
            }
            Err(_) => {
                return Err("账户配置不存在".to_string());
            }
        }
    }




    pub async fn unregister_mt5_exchange(
        &mut self,
        unregister_params: UnregisterMt5ExchangeParams,
    ) -> Result<(), String> {
        tracing::debug!("接收到命令: {:?}", unregister_params);
        // 转换为mt5
        let mt5 = self
            .get_exchange(&unregister_params.terminal_id)
            .await
            .unwrap();
        // 类型转换
        let mt5 = mt5.as_any().downcast_ref::<MetaTrader5>().unwrap();
        // python后端的实例也删除后，再删除exchange_engine的实例
        self.exchanges.remove(&unregister_params.terminal_id);
        Ok(())
    }

    pub async fn is_registered(&self, account_id: &i32) -> bool {
        self.exchanges.contains_key(account_id)
    }

    pub async fn get_exchange(&self, account_id: &i32) -> Result<Box<dyn ExchangeClient>, String> {
        match self.exchanges.get(account_id) {
            Some(client) => {
                // 使用clone_box方法直接获取一个新的Box<dyn ExchangeClient>
                Ok(client.clone_box())
            }
            None => Err(format!("交易所 {:?} 未注册", account_id)),
        }
    }

    pub async fn get_exchange_ref<'a>(
        &'a self,
        account_id: &i32,
    ) -> Result<&'a Box<dyn ExchangeClient>, String> {
        match self.exchanges.get(account_id) {
            Some(client) => Ok(client),
            None => Err(format!("交易所 {:?} 未注册", account_id)),
        }
    }

    // 添加一个获取可变引用的方法
    pub async fn get_exchange_mut<'a>(
        &'a mut self,
        account_id: &i32,
    ) -> Result<&'a mut Box<dyn ExchangeClient>, String> {
        match self.exchanges.get_mut(account_id) {
            Some(client) => Ok(client),
            None => Err(format!("交易所 {:?} 未注册", account_id)),
        }
    }

    // 由exchange_engine启动mt5服务器
    // pub async fn start_mt5_server(
    //     &self,
    //     debug_output: bool,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     // 先检查并清理可能存在的旧进程
    //     #[cfg(windows)]
    //     {
    //         // 查找所有MetaTrader5.exe进程
    //         let output = StdCommand::new("tasklist")
    //             .args(&["/FI", "IMAGENAME eq MetaTrader5.exe", "/FO", "CSV"])
    //             .output()?;

    //         let output_str = String::from_utf8_lossy(&output.stdout);
    //         if output_str.contains("MetaTrader5.exe") {
    //             tracing::warn!("发现旧的MetaTrader5进程, 正在清理...");

    //             // 强制结束所有MetaTrader5.exe进程
    //             let _ = StdCommand::new("taskkill")
    //                 .args(&["/F", "/IM", "MetaTrader5.exe"])
    //                 .output()?;

    //             // 等待进程完全退出
    //             tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    //         }
    //     }
    //     let py_exe = Asset::get("MetaTrader5-x86_64-pc-windows-msvc.exe")
    //         .ok_or("获取python可执行文件失败")?;

    //     let temp_dir = TempDir::new()?;
    //     let exe_path = temp_dir.path().join("MetaTrader5.exe");
    //     fs::write(&exe_path, py_exe.data)?;

    //     // 创建子进程，设置新的进程组
    //     let mut command = Command::new(exe_path);

    //     #[cfg(windows)]
    //     {
    //         command.creation_flags(CREATE_NEW_PROCESS_GROUP.0 as u32);
    //     }

    //     // 添加-u参数禁用Python输出缓冲
    //     command.arg("-u");

    //     // 根据debug_output参数决定如何处理输出
    //     let mut child = if debug_output {
    //         // 直接输出到终端
    //         command
    //             .stdout(Stdio::inherit())
    //             .stderr(Stdio::inherit())
    //             .spawn()?
    //     } else {
    //         // 捕获输出用于日志
    //         command
    //             .stdout(Stdio::piped())
    //             .stderr(Stdio::piped())
    //             .spawn()?
    //     };

    //     // 如果不是直接输出到终端，则捕获输出到日志
    //     if !debug_output {
    //         // 处理标准输出
    //         if let Some(stdout) = child.stdout.take() {
    //             tokio::spawn(async move {
    //                 use tokio::io::{AsyncBufReadExt, BufReader};
    //                 let reader = BufReader::new(stdout);
    //                 let mut lines = reader.lines();

    //                 while let Ok(Some(line)) = lines.next_line().await {
    //                     // 控制mt5的输出
    //                     // tracing::info!("MT5 output: {}", line);
    //                 }
    //             });
    //         }

    //         // 处理标准错误
    //         if let Some(stderr) = child.stderr.take() {
    //             tokio::spawn(async move {
    //                 use tokio::io::{AsyncBufReadExt, BufReader};
    //                 let reader = BufReader::new(stderr);
    //                 let mut lines = reader.lines();

    //                 while let Ok(Some(line)) = lines.next_line().await {
    //                     tracing::warn!("MT5 error: {}", line);
    //                 }
    //             });
    //         }
    //     }

    //     *self.mt5_process.lock().unwrap() = Some(child);
    //     self.is_mt5_server_running
    //         .store(true, std::sync::atomic::Ordering::Relaxed);

    //     tracing::info!("metatrader5服务启动成功");
    //     Ok(())
    // }
}
