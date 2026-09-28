use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::entities::{RespaldoInfo, RespaldoResult, RestauracionResult};
use crate::domain::repositories::RespaldoRepository;
use crate::infrastructure::database::get_db_path;
use crate::infrastructure::error::AppError;
use crate::infrastructure::repositories::SqliteRespaldoRepository;

const EXTENSION_RESPALDO: &str = "db";

pub struct RespaldoService {
    repository: Arc<dyn RespaldoRepository>,
    /// Where the live database lives. Restoring a backup over itself would
    /// truncate the source while reading from it, so the service rejects it
    /// before the repository ever opens the file.
    ruta_actual: PathBuf,
}

impl Default for RespaldoService {
    fn default() -> Self {
        Self::new()
    }
}

impl RespaldoService {
    pub fn new() -> Self {
        Self::with_ruta_actual(Arc::new(SqliteRespaldoRepository::new()), get_db_path())
    }

    pub fn with_repository(repository: Arc<dyn RespaldoRepository>) -> Self {
        Self::with_ruta_actual(repository, get_db_path())
    }

    pub fn with_ruta_actual(repository: Arc<dyn RespaldoRepository>, ruta_actual: PathBuf) -> Self {
        Self {
            repository,
            ruta_actual,
        }
    }

    pub fn crear(&self, destino: &str) -> Result<RespaldoResult, AppError> {
        let ruta = normalizar_destino(destino)?;
        self.repository.crear_respaldo(&ruta)
    }

    pub fn info(&self) -> Result<RespaldoInfo, AppError> {
        self.repository.info()
    }

    pub fn restaurar(&self, origen: &str) -> Result<RestauracionResult, AppError> {
        let ruta = normalizar_origen(origen)?;
        if es_misma_base(&ruta, &self.ruta_actual) {
            return Err(AppError::RespaldoInvalido(
                "es la base de datos que está en uso".to_string(),
            ));
        }
        self.repository.restaurar(&ruta)
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

/// The origin is a file the user just picked, so it is used exactly as chosen.
///
/// Unlike `normalizar_destino` this does not append a `.db` extension: the
/// backup has to exist under the name that was selected, and inventing a
/// different one would turn a real path into a missing one.
fn normalizar_origen(origen: &str) -> Result<PathBuf, AppError> {
    let origen = origen.trim();
    if origen.is_empty() {
        return Err(AppError::RespaldoPathInvalido);
    }

    Ok(PathBuf::from(origen))
}

fn es_misma_base(candidata: &Path, actual: &Path) -> bool {
    let canonica = |ruta: &Path| ruta.canonicalize().unwrap_or_else(|_| ruta.to_path_buf());
    canonica(candidata) == canonica(actual)
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

    fn resultado_restauracion() -> RestauracionResult {
        RestauracionResult {
            ruta_origen: "copia.db".to_string(),
            tablas_restauradas: 21,
            registros_restaurados: 7,
            ruta_respaldo_previo: None,
            restaurado_en: "2026-09-28T10:00:00+00:00".to_string(),
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

    #[test]
    fn restaurar_rechaza_origen_vacio_o_blanco() {
        let mut repo = MockRespaldoRepository::new();
        repo.expect_restaurar().never();
        let service = RespaldoService::with_repository(Arc::new(repo));

        assert!(matches!(
            service.restaurar("   "),
            Err(AppError::RespaldoPathInvalido)
        ));
    }

    #[test]
    fn restaurar_rechaza_la_base_que_esta_en_uso() {
        let mut repo = MockRespaldoRepository::new();
        repo.expect_restaurar().never();
        let service = RespaldoService::with_ruta_actual(
            Arc::new(repo),
            PathBuf::from("/datos/el-kioskon-app.db"),
        );

        assert!(matches!(
            service.restaurar("/datos/el-kioskon-app.db"),
            Err(AppError::RespaldoInvalido(detalle)) if detalle.contains("está en uso")
        ));
    }

    #[test]
    fn restaurar_delega_el_origen_normalizado_en_el_repositorio() {
        let mut repo = MockRespaldoRepository::new();
        repo.expect_restaurar()
            .withf(|origen| origen == Path::new("/copias/mi-copia.db"))
            .returning(|_| Ok(resultado_restauracion()));
        let service = RespaldoService::with_ruta_actual(
            Arc::new(repo),
            PathBuf::from("/datos/el-kioskon-app.db"),
        );

        let resultado = service.restaurar("  /copias/mi-copia.db  ").unwrap();

        assert_eq!(resultado.registros_restaurados, 7);
    }

    #[test]
    fn restaurar_usa_el_archivo_elegido_sin_cambiarle_la_extension() {
        let mut repo = MockRespaldoRepository::new();
        // A backup the user picked that happens to have no extension must be
        // read as chosen; appending `.db` would look for a file that is not
        // there.
        repo.expect_restaurar()
            .withf(|origen| origen == Path::new("/copias/mi-copia"))
            .returning(|_| Ok(resultado_restauracion()));
        let service = RespaldoService::with_ruta_actual(
            Arc::new(repo),
            PathBuf::from("/datos/el-kioskon-app.db"),
        );

        assert!(service.restaurar("/copias/mi-copia").is_ok());
    }

    #[test]
    fn restaurar_propaga_el_error_del_repositorio() {
        let mut repo = MockRespaldoRepository::new();
        repo.expect_restaurar()
            .returning(|_| Err(AppError::RestaurarRespaldoError("sin permiso".to_string())));
        let service = RespaldoService::with_repository(Arc::new(repo));

        assert!(matches!(
            service.restaurar("copia.db"),
            Err(AppError::RestaurarRespaldoError(_))
        ));
    }

    #[test]
    fn es_misma_base_compara_resueltas_por_el_sistema() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("el-kioskon-app.db");
        std::fs::write(&ruta, b"base").unwrap();
        let indirecta = dir.path().join(".").join("el-kioskon-app.db");

        assert!(es_misma_base(&ruta, &indirecta));
        assert!(!es_misma_base(&ruta, &dir.path().join("otra-copia.db")));
    }

    #[test]
    fn es_misma_base_no_falla_si_la_ruta_no_existe() {
        let dir = tempfile::tempdir().unwrap();
        let una = dir.path().join("primera.db");
        let otra = dir.path().join("segunda.db");

        // Neither path resolves, so the comparison falls back to the raw paths
        // instead of treating every unresolvable path as the same file.
        assert!(!es_misma_base(&una, &otra));
        assert!(es_misma_base(
            &una,
            &dir.path().join(".").join("primera.db")
        ));
        assert!(!es_misma_base(&una, Path::new(":memory:")));
    }
}
