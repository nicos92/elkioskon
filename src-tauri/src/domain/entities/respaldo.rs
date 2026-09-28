use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RespaldoResult {
    pub ruta: String,
    pub tamano_bytes: u64,
    pub generado_en: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RespaldoInfo {
    pub ruta_base_datos: String,
    pub tamano_base_datos_bytes: u64,
}

/// Outcome of restoring a backup over the live database.
///
/// `ruta_respaldo_previo` points at the safety copy taken of the data that was
/// replaced, so the operation can be undone by hand. It is `None` when the
/// live database is not file backed (tests).
#[derive(Debug, Clone, Serialize)]
pub struct RestauracionResult {
    pub ruta_origen: String,
    pub tablas_restauradas: usize,
    pub registros_restaurados: u64,
    pub ruta_respaldo_previo: Option<String>,
    pub restaurado_en: String,
}
