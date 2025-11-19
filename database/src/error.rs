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
}

// Implement the StarRiverErrorTrait for StarRiverError
impl StarRiverErrorTrait for DatabaseError {
    fn get_prefix(&self) -> &'static str {
        "Database"
    }

    fn error_code(&self) -> ErrorCode {
        let prefix = self.get_prefix();
        let code = match self {
            DatabaseError::SeaOrmError { .. } => 1001,            // sea-orm error
        };
        format!("{}_{:04}", prefix, code)
    }

    fn http_status_code(&self) -> star_river_core::error::StatusCode {
        match self {
            DatabaseError::SeaOrmError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_message(&self, language: ErrorLanguage) -> String {
        match language {
            ErrorLanguage::English => self.to_string(),
            ErrorLanguage::Chinese => match self {
                DatabaseError::SeaOrmError { source, .. } => {
                    format!("数据库错误: {}", source)
                }
            },
        }
    }

    fn error_code_chain(&self) -> Vec<ErrorCode> {
        match self {
            DatabaseError::SeaOrmError { .. } => vec![self.error_code()],
        }
    }
}
