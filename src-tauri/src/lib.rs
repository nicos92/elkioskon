pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

use api::commands::{
    actualizar_cliente, add_permission_to_user, anular_venta, apply_costo_percentage_stock,
    cambiar_estado_presupuesto, change_password, cleanup_cost_update_operations, crear_cierre,
    crear_cliente, crear_presupuesto, create_articulo, create_categoria, create_permission,
    create_proveedor, create_stock, create_sub_categoria, create_tipo_venta, create_user,
    create_venta, delete_articulo, delete_categoria, delete_dollar_quote, delete_proveedor,
    delete_stock, delete_sub_categoria, delete_tipo_venta, delete_user, eliminar_cliente,
    ensure_db_ready, fetch_dollar_rates_manual, get_all_articulos, get_all_categorias,
    get_all_cierres, get_all_clientes, get_all_permissions, get_all_presupuestos,
    get_all_proveedores, get_all_stock, get_all_sub_categorias, get_all_tipos_venta, get_all_users,
    get_all_ventas, get_audit_logs, get_cliente_by_id, get_cliente_defecto, get_dollar_quotes,
    get_home_stats, get_last_undoable_cost_update, get_precio_venta, get_presupuesto_by_id,
    get_proveedor_by_id, get_stock_by_articulo, get_stock_by_id, get_stock_preview_costo,
    get_sub_categorias_by_categoria, get_user_permissions, get_venta_by_id, get_ventas_por_cliente,
    is_dia_cerrado, login, reabrir_cierre, remove_permission_from_user, undo_cost_update,
    update_articulo, update_categoria, update_proveedor, update_stock, update_sub_categoria,
    update_tipo_venta, update_user, AppState, ArticuloAppState, AuditLogAppState,
    CategoriaAppState, CierreAppState, ClienteAppState, CostUpdateAppState, DollarAppState,
    HomeStatsAppState, PresupuestoAppState, ProveedorAppState, StockAppState, SubCategoriaAppState,
    TipoVentaAppState, VentaAppState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new())
        .manage(AuditLogAppState::new())
        .manage(HomeStatsAppState::new())
        .manage(ProveedorAppState::new())
        .manage(CategoriaAppState::new())
        .manage(SubCategoriaAppState::new())
        .manage(ArticuloAppState::new())
        .manage(StockAppState::new())
        .manage(CostUpdateAppState::new())
        .manage(VentaAppState::new())
        .manage(PresupuestoAppState::new())
        .manage(TipoVentaAppState::new())
        .manage(CierreAppState::new())
        .manage(ClienteAppState::new())
        .manage(DollarAppState::new())
        .invoke_handler(tauri::generate_handler![
            ensure_db_ready,
            get_home_stats,
            login,
            create_user,
            get_all_users,
            update_user,
            change_password,
            delete_user,
            add_permission_to_user,
            remove_permission_from_user,
            get_user_permissions,
            get_all_permissions,
            create_permission,
            get_all_proveedores,
            get_proveedor_by_id,
            create_proveedor,
            update_proveedor,
            delete_proveedor,
            get_all_categorias,
            create_categoria,
            update_categoria,
            delete_categoria,
            get_all_sub_categorias,
            get_sub_categorias_by_categoria,
            create_sub_categoria,
            update_sub_categoria,
            delete_sub_categoria,
            get_all_articulos,
            create_articulo,
            update_articulo,
            delete_articulo,
            get_all_stock,
            get_stock_by_id,
            get_stock_by_articulo,
            create_stock,
            update_stock,
            delete_stock,
            get_precio_venta,
            get_stock_preview_costo,
            apply_costo_percentage_stock,
            get_last_undoable_cost_update,
            undo_cost_update,
            cleanup_cost_update_operations,
            get_audit_logs,
            create_venta,
            get_all_ventas,
            get_venta_by_id,
            get_ventas_por_cliente,
            anular_venta,
            get_all_tipos_venta,
            create_tipo_venta,
            update_tipo_venta,
            delete_tipo_venta,
            crear_cierre,
            get_all_cierres,
            reabrir_cierre,
            is_dia_cerrado,
            get_all_clientes,
            get_cliente_by_id,
            get_cliente_defecto,
            crear_cliente,
            actualizar_cliente,
            eliminar_cliente,
            get_dollar_quotes,
            fetch_dollar_rates_manual,
            delete_dollar_quote,
            crear_presupuesto,
            get_all_presupuestos,
            get_presupuesto_by_id,
            cambiar_estado_presupuesto
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
