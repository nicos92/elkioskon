use std::sync::Arc;

use crate::domain::entities::{AuditAction, AuditLog, AuditScreen};
use crate::domain::repositories::{AuditLogFilter, AuditLogRepository, Page};
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::SqliteAuditLogRepository;

pub struct AuditLogService {
    repository: Arc<dyn AuditLogRepository>,
}

impl Default for AuditLogService {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLogService {
    pub fn new() -> Self {
        Self::with_repository(Arc::new(SqliteAuditLogRepository::new()))
    }

    pub fn with_repository(repository: Arc<dyn AuditLogRepository>) -> Self {
        Self { repository }
    }

    pub fn log(
        &self,
        user_id: i64,
        screen: AuditScreen,
        action: AuditAction,
        detail: Option<String>,
    ) -> Result<AuditLog, AppError> {
        let username = self.repository.get_username(user_id)?.unwrap_or_default();
        let log = AuditLog::new(user_id, username, screen, action, detail);
        self.repository.create(&log)
    }

    pub fn get_logs(&self, filter: &AuditLogFilter) -> Result<Page<AuditLog>, AppError> {
        self.repository.find_with_filters(filter)
    }
}

pub fn log_audit(
    user_id: i64,
    screen: AuditScreen,
    action: AuditAction,
    detail: Option<String>,
) -> Result<AuditLog, AppError> {
    let service = AuditLogService::new();
    service.log(user_id, screen, action, detail)
}
