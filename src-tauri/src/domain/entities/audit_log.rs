use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Create,
    Update,
    Read,
    Delete,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Create => "nuevo",
            AuditAction::Update => "modificar",
            AuditAction::Read => "consultar",
            AuditAction::Delete => "eliminar",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditScreen {
    Usuarios,
    Proveedores,
    Clientes,
    Categorias,
    SubCategorias,
    Articulos,
    Stock,
    Ventas,
    Presupuestos,
    TiposVenta,
    Permisos,
    Auditoria,
    Cierres,
    Dolar,
    Configuracion,
}

impl AuditScreen {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditScreen::Usuarios => "Usuarios",
            AuditScreen::Proveedores => "Proveedores",
            AuditScreen::Clientes => "Clientes",
            AuditScreen::Categorias => "Categorias",
            AuditScreen::SubCategorias => "SubCategorias",
            AuditScreen::Articulos => "Articulos",
            AuditScreen::Stock => "Stock",
            AuditScreen::Ventas => "Ventas",
            AuditScreen::Presupuestos => "Presupuestos",
            AuditScreen::TiposVenta => "Tipos de Venta",
            AuditScreen::Permisos => "Permisos",
            AuditScreen::Auditoria => "Auditoria",
            AuditScreen::Cierres => "Cierres del día",
            AuditScreen::Dolar => "Dólar",
            AuditScreen::Configuracion => "Configuración",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditLog {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub screen: String,
    pub action: String,
    pub detail: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    pub fn new(
        user_id: i64,
        username: String,
        screen: AuditScreen,
        action: AuditAction,
        detail: Option<String>,
    ) -> Self {
        Self {
            id: 0,
            user_id,
            username,
            screen: screen.as_str().to_string(),
            action: action.as_str().to_string(),
            detail,
            created_at: Utc::now(),
        }
    }
}
