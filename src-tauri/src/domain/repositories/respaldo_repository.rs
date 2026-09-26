use std::path::Path;

use crate::domain::entities::{RespaldoInfo, RespaldoResult};
use crate::infrastructure::error::AppError;

#[cfg_attr(test, mockall::automock)]
pub trait RespaldoRepository: Send + Sync {
    fn crear_respaldo(&self, destino: &Path) -> Result<RespaldoResult, AppError>;
    fn info(&self) -> Result<RespaldoInfo, AppError>;
}
