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
