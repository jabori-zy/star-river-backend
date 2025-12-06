use sea_orm::DbErr;
use snafu::{Backtrace, Snafu};
use star_river_core::error::{
    ErrorCode, StatusCode,
    error_trait::{ErrorLanguage, StarRiverErrorTrait},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum DatabaseError {
    #[snafu(transparent)]
    SeaOrmError { source: DbErr, backtrace: Backtrace },

    #[snafu(display("home directory not found: {source}"))]
    HomeDirNotFound { source: std::env::VarError, backtrace: Backtrace },

    #[snafu(display("create directory failed: {dir}: {source}"))]
    DirCreateFailed {
        dir: String,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("work directory not found: {source}"))]
    WorkDirNotFound { source: std::io::Error, backtrace: Backtrace },
}

// Implement the StarRiverErrorTrait for StarRiverError
impl StarRiverErrorTrait for DatabaseError {
    fn get_prefix(&self) -> &'static str {
        "Database"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            DatabaseError::SeaOrmError { .. } => 1001,     // sea-orm error
            DatabaseError::HomeDirNotFound { .. } => 1002, // home directory not found
            DatabaseError::DirCreateFailed { .. } => 1003, // create directory failed
            DatabaseError::WorkDirNotFound { .. } => 1004, // work directory not found
        };
        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> star_river_core::error::StatusCode {
        match self {
            DatabaseError::SeaOrmError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            DatabaseError::HomeDirNotFound { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            DatabaseError::DirCreateFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            DatabaseError::WorkDirNotFound { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                DatabaseError::SeaOrmError { source, .. } => {
                    format!("数据库错误: {}", source)
                }
                DatabaseError::HomeDirNotFound { source, .. } => {
                    format!("home 文件夹未找到: {}", source)
                }
                DatabaseError::DirCreateFailed { dir, source, .. } => {
                    format!("创建目录失败: {}: {}", dir, source)
                }
                DatabaseError::WorkDirNotFound { source, .. } => {
                    format!("工作目录未找到: {}", source)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            DatabaseError::SeaOrmError { .. }
            | DatabaseError::HomeDirNotFound { .. }
            | DatabaseError::DirCreateFailed { .. }
            | DatabaseError::WorkDirNotFound { .. } => vec![self.error_code()],
        }
    }
}
