pub mod entities;

use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use std::path::PathBuf;
use sea_orm_migration::MigratorTrait;
use migration::Migrator;





pub struct DatabaseManager {
    pub path: PathBuf,
    pub conn: DatabaseConnection,
}

impl DatabaseManager {

    pub async fn new() -> Self {
        let path = Self::get_database_path().unwrap();
        // 初始化数据库
        let conn = Self::create_database(&path).await.unwrap();
        Self { path, conn }
    }

    pub async fn migrate(&self) {
        Migrator::up(&self.conn, None).await.unwrap();
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
        let db_path = path.join("db.sqlite");
        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let conn = Database::connect(database_url).await.unwrap();
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
