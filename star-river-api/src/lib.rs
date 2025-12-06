// Star River API Library
// Pure API definition, does not contain any business startup logic

// Internal modules
mod engine_manager;
mod websocket;

// Public modules - for external use
pub mod api;
pub mod error;
pub mod routes;
pub mod sse;
pub mod star_river;
// Re-export commonly used types
pub use engine_manager::EngineManager;
pub use star_river::{StarRiver, init_app};
