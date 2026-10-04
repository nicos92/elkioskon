use std::sync::Mutex;

use crate::application::services::UserService;
use crate::infrastructure::error::AppError;

pub mod articulo_commands;
pub mod audit_log_commands;
pub mod categoria_commands;
pub mod cierre_commands;
pub mod cliente_commands;
pub mod cost_update_commands;
pub mod dollar_commands;
pub mod home_commands;
pub mod nocturno_commands;
pub mod permission_commands;
pub mod permissions;
pub mod presupuesto_commands;
pub mod proveedor_commands;
pub mod respaldo_commands;
pub mod stock_commands;
pub mod sub_categoria_commands;
pub mod tipo_venta_commands;
pub mod user_commands;
pub mod venta_commands;

pub use articulo_commands::{
    create_articulo, delete_articulo, get_all_articulos, update_articulo, ArticuloAppState,
};
pub use audit_log_commands::{get_audit_logs, AuditLogAppState};
pub use categoria_commands::{
    create_categoria, delete_categoria, get_all_categorias, update_categoria, CategoriaAppState,
};
pub use cierre_commands::{
    crear_cierre, get_all_cierres, is_dia_cerrado, reabrir_cierre, CierreAppState,
};
pub use cliente_commands::{
    actualizar_cliente, crear_cliente, eliminar_cliente, get_all_clientes, get_cliente_by_id,
    get_cliente_defecto, ClienteAppState,
};
pub use cost_update_commands::{
    apply_costo_percentage_stock, cleanup_cost_update_operations, get_last_undoable_cost_update,
    get_stock_preview_costo, undo_cost_update, CostUpdateAppState,
};
pub use dollar_commands::{
    delete_dollar_quote, fetch_dollar_rates_manual, get_dollar_quotes, DollarAppState,
};
pub use home_commands::{get_home_stats, HomeStatsAppState};
pub use nocturno_commands::{get_nocturno_config, save_nocturno_config, NocturnoConfigAppState};
pub use permission_commands::{
    add_permission_to_user, create_permission, get_all_permissions, get_user_permissions,
    remove_permission_from_user, AddPermissionRequest,
};
pub use presupuesto_commands::{
    cambiar_estado_presupuesto, crear_presupuesto, get_all_presupuestos, get_presupuesto_by_id,
    PresupuestoAppState,
};
pub use proveedor_commands::{
    create_proveedor, delete_proveedor, get_all_proveedores, get_proveedor_by_id, update_proveedor,
    ProveedorAppState,
};
pub use respaldo_commands::{
    crear_respaldo, get_respaldo_info, restaurar_respaldo, RespaldoAppState,
};
pub use stock_commands::{
    create_stock, delete_stock, get_all_stock, get_precio_venta, get_stock_by_articulo,
    get_stock_by_id, update_stock, StockAppState,
};
pub use sub_categoria_commands::{
    create_sub_categoria, delete_sub_categoria, get_all_sub_categorias,
    get_sub_categorias_by_categoria, update_sub_categoria, SubCategoriaAppState,
};
pub use tipo_venta_commands::{
    create_tipo_venta, delete_tipo_venta, get_all_tipos_venta, update_tipo_venta, TipoVentaAppState,
};
pub use user_commands::{
    change_password, create_user, delete_user, get_all_users, login, update_user,
    ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, UpdateUserRequest,
    UserResponse,
};
pub use venta_commands::{
    anular_venta, create_venta, get_all_ventas, get_venta_by_id, get_ventas_por_cliente,
    VentaAppState,
};

pub struct AppState {
    pub user_service: Mutex<UserService>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user_service: Mutex::new(UserService::new()),
        }
    }
}

#[tauri::command]
pub async fn ensure_db_ready() -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(|| {
        let _ = &*crate::infrastructure::database::DB;
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))
}
