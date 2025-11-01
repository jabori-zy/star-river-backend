// Star River API 库
// 纯 API 定义，不包含任何业务启动逻辑

// 内部模块
mod engine_manager;
mod websocket;

// 公开模块 - 供外部使用
pub mod api;
pub mod routes;
pub mod sse;
pub mod star_river;

// 重新导出常用类型
pub use engine_manager::EngineManager;
pub use star_river::{StarRiver, init_app};
