use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::entities::{RespaldoInfo, RespaldoResult};
use crate::domain::repositories::RespaldoRepository;
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::SqliteRespaldoRepository;

const EXTENSION_RESPALDO: &str = "db";

pub struct RespaldoService {
    repository: Arc<dyn RespaldoRepository>,
}

impl Default for RespaldoService {
    fn default() -> Self {
        Self::new()
    }
}

impl RespaldoService {
    pub fn new() -> Self {
        Self::with_repository(Arc::new(SqliteRespaldoRepository::new()))
    }

    pub fn with_repository(repository: Arc<dyn RespaldoRepository>) -> Self {
        Self { repository }
    }

    pub fn crear(&self, destino: &str) -> Result<RespaldoResult, AppError> {
        let ruta = normalizar_destino(destino)?;
        self.repository.crear_respaldo(&ruta)
    }

    pub fn info(&self) -> Result<RespaldoInfo, AppError> {
        self.repository.info()
    }
}

fn normalizar_destino(destino: &str) -> Result<PathBuf, AppError> {
    let destino = destino.trim();
    if destino.is_empty() {
        return Err(AppError::RespaldoPathInvalido);
    }

    let ruta = Path::new(destino);
    if ruta.extension().is_none() {
        return Ok(ruta.with_extension(EXTENSION_RESPALDO));
    }

    Ok(ruta.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repositories::respaldo_repository::MockRespaldoRepository;

    fn resultado_respaldo() -> RespaldoResult {
        RespaldoResult {
            ruta: "copia.db".to_string(),
            tamano_bytes: 10,
            generado_en: "2026-09-26T10:00:00+00:00".to_string(),
        }
    }

    #[test]
    fn crear_agrega_la_extension_db_sin_extension() {
        let resultado = normalizar_destino("C:/respaldos/mi-copia");
        assert_eq!(
            resultado.unwrap(),
            PathBuf::from("C:/respaldos/mi-copia.db")
        );
    }

    #[test]
    fn crear_conserva_la_extension_ya_presente() {
        let resultado = normalizar_destino("C:/respaldos/mi-copia.sqlite");
        assert_eq!(
            resultado.unwrap(),
            PathBuf::from("C:/respaldos/mi-copia.sqlite")
        );
    }

    #[test]
    fn crear_rechaza_destino_vacio_o_blanco() {
        assert!(matches!(
            normalizar_destino("   "),
            Err(AppError::RespaldoPathInvalido)
        ));
    }

    #[test]
    fn crear_delega_el_destino_normalizado_en_el_repositorio() {
        let mut repo = MockRespaldoRepository::new();
        repo.expect_crear_respaldo()
            .withf(|destino| destino == Path::new("copia.db"))
            .returning(|_| Ok(resultado_respaldo()));
        let service = RespaldoService::with_repository(Arc::new(repo));

        let resultado = service.crear("  copia  ").unwrap();

        assert_eq!(resultado.tamano_bytes, 10);
    }

    #[test]
    fn crear_propaga_el_error_del_repositorio() {
        let mut repo = MockRespaldoRepository::new();
        repo.expect_crear_respaldo()
            .returning(|_| Err(AppError::RespaldoError("sin espacio".to_string())));
        let service = RespaldoService::with_repository(Arc::new(repo));

        assert!(matches!(
            service.crear("copia.db"),
            Err(AppError::RespaldoError(_))
        ));
    }

    #[test]
    fn crear_no_llama_al_repositorio_con_destino_invalido() {
        let mut repo = MockRespaldoRepository::new();
        repo.expect_crear_respaldo().never();
        let service = RespaldoService::with_repository(Arc::new(repo));

        assert!(matches!(
            service.crear(""),
            Err(AppError::RespaldoPathInvalido)
        ));
    }
}
