mod server;

use axum::extract::State;
use clap::{Parser, ValueEnum};
use star_river_api::{
    routes::create_app_routes,
    star_river::{StarRiver, init_app},
};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Info,
    Debug,
    Warn,
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Warn => tracing::Level::WARN,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "star-river")]
#[command(about = "Star River Backend Server")]
struct Args {
    /// Server port
    #[arg(short, long, default_value_t = 3100)]
    port: u16,

    /// Log level for stdout
    #[arg(short, long, value_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Load environment variables from .env file
    let _ = dotenvy::dotenv();

    // Initialize logging
    server::init_logging(args.log_level.into())?;

    // Create CORS configuration
    let cors = server::create_cors();

    // Create application state
    let state = StarRiver::new().await;

    // Create routes
    let app = create_app_routes(state.clone()).layer(cors);

    // Initialize application
    init_app(State(state)).await;

    // Build listener address
    let addr = format!("0.0.0.0:{}", args.port).parse()?;

    // Start server
    server::serve(app, addr).await
}
