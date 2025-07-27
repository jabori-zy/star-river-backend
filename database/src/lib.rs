
pub mod mutation;
pub mod query;



use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::env;
use std::path::PathBuf;
use sea_orm_migration::MigratorTrait;
use migration::Migrator;
use log::LevelFilter;


#[derive(Debug)]
pub struct DatabaseManager {
    pub path: PathBuf,
    pub conn: DatabaseConnection,
}

impl DatabaseManager {

    pub async fn new() -> Self {
        let path = Self::get_database_path().unwrap();
        tracing::info!("get_database_path数据库路径: {}", path.display());
        // 初始化数据库
        let conn = Self::create_database(&path).await.unwrap();
        Self { path, conn }
    }

    pub async fn migrate(conn: &DatabaseConnection) {
        let pending_migrations = Migrator::get_pending_migrations(conn).await.unwrap();
        if !pending_migrations.is_empty() {
            tracing::info!("发现 {} 个待应用的迁移", pending_migrations.len());
            
            // 应用迁移
            Migrator::up(conn, None).await.unwrap();
            tracing::info!("所有迁移已成功应用");
        } else {
            tracing::info!("数据库已是最新版本，无需迁移");
        }
    }

    pub fn get_conn(&self) -> DatabaseConnection {
        self.conn.clone()
    }


    // 获取数据库路径
    fn get_database_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // 在开发模式下，数据库文件位于工作目录
        #[cfg(debug_assertions)]
        {
            let run_path = env::current_dir()?;
            let db_path = PathBuf::from(&run_path).join("db");
            // 检查是否存在db目录,如果没有则创建
            if !db_path.exists() {
                std::fs::create_dir_all(&db_path)?;
            }

            Ok(db_path)
        }
        #[cfg(not(debug_assertions))]
        {
            let exe_path = env::current_exe()?;
            let db_path = PathBuf::from(&exe_path);
            Ok(db_path)
        }


    }

    pub async fn create_database(path: &PathBuf) -> Result<DatabaseConnection, DbErr> {
        // 创建数据库文件
        // let db_path = path.join("db.sqlite");
        // let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let path = PathBuf::from("E:/project/star-river-backend/db/db.sqlite");
        tracing::info!("path: {}", path.display());
        let database_url = format!("sqlite:{}?mode=rwc", path.display());
        tracing::info!("数据库路径: {}", database_url);
        
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