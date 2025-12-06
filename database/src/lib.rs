pub mod error;
pub mod mutation;
pub mod page;
pub mod query;

use std::{env, path::PathBuf};

use log::LevelFilter;
use migration::Migrator;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use snafu::{IntoError, ResultExt};

use crate::error::{DatabaseError, DirCreateFailedSnafu, HomeDirNotFoundSnafu, WorkDirNotFoundSnafu};

#[derive(Debug)]
pub struct DatabaseManager {
    pub path: PathBuf,
    pub conn: DatabaseConnection,
}

impl DatabaseManager {
    pub async fn new() -> Self {
        let path = Self::get_database_path().unwrap();
        // Initialize database
        let conn = Self::create_database(&path).await.unwrap();
        Self { path, conn }
    }

    /// Create in-memory database (for testing)
    ///
    /// Each call creates a fresh, independent in-memory database instance.
    /// Data is automatically destroyed when connection closes, no manual cleanup needed.
    pub async fn new_in_memory() -> Result<Self, DbErr> {
        let database_url = "sqlite::memory:";

        let mut opt = ConnectOptions::new(database_url);
        opt.sqlx_logging(false).sqlx_logging_level(LevelFilter::Debug);

        let conn = Database::connect(opt).await?;

        // Apply all migrations to ensure schema matches production environment
        Self::migrate(&conn).await;

        Ok(Self {
            path: PathBuf::from(":memory:"),
            conn,
        })
    }

    pub async fn migrate(conn: &DatabaseConnection) {
        let pending_migrations = Migrator::get_pending_migrations(conn).await.unwrap();
        if !pending_migrations.is_empty() {
            tracing::info!("found {} pending migrations", pending_migrations.len());

            // Apply migrations
            Migrator::up(conn, None).await.unwrap();
            tracing::info!("all migrations applied successfully");
        } else {
            tracing::info!("database is up to date, no migrations needed");
        }
    }

    pub fn get_conn(&self) -> DatabaseConnection {
        self.conn.clone()
    }

    // Get database path
    fn get_database_path() -> Result<PathBuf, DatabaseError> {
        // In development mode, database file is located in working directory
        #[cfg(debug_assertions)]
        {
            // Find workspace root directory (directory containing top-level Cargo.toml)
            let root_path = Self::find_workspace_root()?;
            let db_path = root_path.join("db");
            tracing::info!("dev environment database path: {}", db_path.display());

            // Check if db directory exists, create if not
            if !db_path.exists() {
                std::fs::create_dir_all(&db_path).context(DirCreateFailedSnafu {
                    dir: db_path.display().to_string(),
                })?;
            }

            Ok(db_path)
        }
        #[cfg(not(debug_assertions))]
        {
            #[cfg(target_os = "macos")]
            {
                Self::get_darwin_database_path()
            }

            #[cfg(target_os = "windows")]
            {
                Self::get_windows_database_path()
            }
        }
    }

    /// Get database path on macOS system
    /// Database is stored in ~/Library/Application Support/StarRiver/ directory
    /// Prevents user data from being overwritten during app updates
    #[allow(unused)]
    #[cfg(target_os = "macos")]
    fn get_darwin_database_path() -> Result<PathBuf, DatabaseError> {
        let home_dir = env::var("HOME").context(HomeDirNotFoundSnafu {})?;

        let app_support_path = PathBuf::from(home_dir)
            .join("Library")
            .join("Application Support")
            .join("Star River")
            .join("app_data");

        // Create directory if it doesn't exist
        if !app_support_path.exists() {
            use crate::error::DirCreateFailedSnafu;
            std::fs::create_dir_all(&app_support_path).context(DirCreateFailedSnafu {
                dir: app_support_path.display().to_string(),
            })?;
        }

        tracing::info!("macOS database path: {}", app_support_path.display());
        Ok(app_support_path)
    }

    #[allow(unused)]
    #[cfg(target_os = "windows")]
    fn get_windows_database_path() -> Result<PathBuf, DatabaseError> {
        let app_data = env::var("APPDATA").context(HomeDirNotFoundSnafu {})?;

        let app_data_path = PathBuf::from(app_data)
            .join("Star River")
            .join("app_data");

        // Create directory if it doesn't exist
        if !app_data_path.exists() {
            std::fs::create_dir_all(&app_data_path).context(DirCreateFailedSnafu {
                dir: app_data_path.display().to_string(),
            })?;
        }

        tracing::info!("Windows database path: {}", app_data_path.display());
        Ok(app_data_path)
    }

    // find workspace root directory
    fn find_workspace_root() -> Result<PathBuf, DatabaseError> {
        // First try to find from CARGO_MANIFEST_DIR upwards (at compile time)
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            let mut current = PathBuf::from(manifest_dir);
            while let Some(parent) = current.parent() {
                let cargo_toml = parent.join("Cargo.toml");
                if cargo_toml.exists() {
                    // Check if it's the workspace root (contains [workspace] or db directory exists)
                    if parent.join("db").exists() || Self::is_workspace_root(&cargo_toml) {
                        return Ok(parent.to_path_buf());
                    }
                }
                current = parent.to_path_buf();
            }
        }

        // If above fails, search upwards from current working directory
        let mut current = env::current_dir().context(WorkDirNotFoundSnafu {})?;
        loop {
            let cargo_toml = current.join("Cargo.toml");
            if cargo_toml.exists() {
                // Check if it's the workspace root
                if current.join("db").exists() || Self::is_workspace_root(&cargo_toml) {
                    return Ok(current);
                }
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }
        return Err(WorkDirNotFoundSnafu {}.into_error(std::io::Error::new(std::io::ErrorKind::NotFound, "work directory not found")));
    }

    // check if Cargo.toml contains workspace configuration
    fn is_workspace_root(cargo_toml: &PathBuf) -> bool {
        if let Ok(content) = std::fs::read_to_string(cargo_toml) {
            content.contains("[workspace]")
        } else {
            false
        }
    }

    pub async fn create_database(path: &PathBuf) -> Result<DatabaseConnection, DbErr> {
        // Create database file
        let db_path = path.join("db.sqlite");
        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());

        tracing::info!("database url: {}", database_url);
        let mut opt = ConnectOptions::new(database_url);
        opt.sqlx_logging(false).sqlx_logging_level(LevelFilter::Debug);
        let conn = Database::connect(opt).await.unwrap();
        Self::migrate(&conn).await;
        Ok(conn)
    }

    pub async fn check_connection(&self) -> Result<(), DbErr> {
        self.conn.ping().await?;
        Ok(())
    }

    pub async fn close(self) {
        self.conn.close().await.unwrap();
    }
}
