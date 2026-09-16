use crate::domain::entities::NocturnoConfig;
use crate::infrastructure::error::AppError;

#[cfg_attr(test, mockall::automock)]
pub trait NocturnoConfigRepository: Send + Sync {
    fn get_config(&self) -> Result<NocturnoConfig, AppError>;
    fn save_config(&self, config: &NocturnoConfig) -> Result<(), AppError>;
}