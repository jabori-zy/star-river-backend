pub mod entities;
pub mod mutation;
pub mod query;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::env;
use std::path::PathBuf;
use sea_orm_migration::MigratorTrait;
use migration::Migrator;
use event_center::{Event, EventPublisher};
use event_center::request_event::CommandEvent;
use event_center::request_event::DatabaseCommand;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use log::LevelFilter;



pub struct DatabaseManager {
    pub path: PathBuf,
    pub conn: DatabaseConnection,
    event_publisher: EventPublisher,
    command_event_receiver: broadcast::Receiver<Event>,
}

impl DatabaseManager {

    pub async fn new(
        command_event_receiver: broadcast::Receiver<Event>,
        event_publisher: EventPublisher,
    ) -> Self {
        let path = Self::get_database_path().unwrap();
        // 初始化数据库
        let conn = Self::create_database(&path).await.unwrap();
        Self { path, conn, command_event_receiver, event_publisher }
    }

    pub async fn migrate(&self) {
        Migrator::up(&self.conn, None).await.unwrap();
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
        let path = PathBuf::from("D:/project/star-river-backend/db/db.sqlite");
        let database_url = format!("sqlite:{}?mode=rwc", path.display());
        tracing::info!("数据库路径: {}", database_url);
        let mut opt = ConnectOptions::new(database_url);
        opt.sqlx_logging(false).sqlx_logging_level(LevelFilter::Debug);
        let conn = Database::connect(opt).await.unwrap();
        Ok(conn)
    }

    pub async fn check_connection(&self) -> Result<(), DbErr> {
        self.conn.ping().await?;
        Ok(())
    }

    pub async fn close(self) {
        self.conn.close().await.unwrap();
    }

    pub async fn listen(
        &self,
        internal_tx: mpsc::Sender<Event>,
    ) {
        tracing::info!("数据库启动成功, 开始监听...");
        let mut command_receiver = self.command_event_receiver.resubscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(event) = command_receiver.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                }
            }
        });
    }

    async fn handle_events(mut internal_rx: mpsc::Receiver<Event>) {
        tokio::spawn(async move {
            loop {
                let event = internal_rx.recv().await.unwrap();
                match event {
                    Event::Command(CommandEvent::Database(database_cmd)) => {
                        match database_cmd {
                            DatabaseCommand::CreateStrategy(create_strategy_params) => {
                                tracing::info!("创建策略: {:?}", create_strategy_params);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });       
        
    }
    pub async fn start(&self) {
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        self.listen(internal_tx).await;

        Self::handle_events(internal_rx).await;
    }
}