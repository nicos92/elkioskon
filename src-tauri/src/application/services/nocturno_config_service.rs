use std::sync::Arc;

use crate::domain::entities::{HoraConfig, NocturnoConfig};
use crate::domain::repositories::NocturnoConfigRepository;
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::SqliteNocturnoConfigRepository;

pub struct NocturnoConfigService {
    repository: Arc<dyn NocturnoConfigRepository>,
}

impl Default for NocturnoConfigService {
    fn default() -> Self {
        Self::new()
    }
}

impl NocturnoConfigService {
    pub fn new() -> Self {
        Self::with_repository(Arc::new(SqliteNocturnoConfigRepository::new()))
    }

    pub fn with_repository(repository: Arc<dyn NocturnoConfigRepository>) -> Self {
        Self { repository }
    }

    pub fn get(&self) -> Result<NocturnoConfig, AppError> {
        self.repository.get_config()
    }

    pub fn save(&self, config: &NocturnoConfig) -> Result<(), AppError> {
        validate_config(config)?;
        self.repository.save_config(config)
    }
}

fn validate_config(config: &NocturnoConfig) -> Result<(), AppError> {
    let inicio = HoraConfig::from_hhmm(&config.hora_inicio)
        .ok_or(AppError::RecargoNocturnoInvalido)?;
    let fin = HoraConfig::from_hhmm(&config.hora_fin).ok_or(AppError::RecargoNocturnoInvalido)?;

    if inicio.minutos_desde_medianoche() == fin.minutos_desde_medianoche() {
        return Err(AppError::RecargoNocturnoInvalido);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::nocturno_config::es_horario_nocturno;

    #[test]
    fn save_rejects_formato_hora_invalido() {
        let service = NocturnoConfigService::new();
        let config = NocturnoConfig {
            hora_inicio: "25:00".to_string(),
            ..Default::default()
        };
        assert!(matches!(
            service.save(&config),
            Err(AppError::RecargoNocturnoInvalido)
        ));
    }

    #[test]
    fn save_rejects_rango_inicio_igual_fin() {
        let service = NocturnoConfigService::new();
        let config = NocturnoConfig {
            hora_inicio: "22:00".to_string(),
            hora_fin: "22:00".to_string(),
            ..Default::default()
        };
        assert!(matches!(
            service.save(&config),
            Err(AppError::RecargoNocturnoInvalido)
        ));
    }

    #[test]
    fn helper_nocturno_cruza_la_medianoche() {
        assert!(es_horario_nocturno(23 * 60, 22 * 60, 6 * 60));
        assert!(es_horario_nocturno(2 * 60, 22 * 60, 6 * 60));
        assert!(!es_horario_nocturno(10 * 60, 22 * 60, 6 * 60));
    }
}