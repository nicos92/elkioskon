use rusqlite::Connection;

const PERMISSIONS: &[&str] = &[
    // Usuarios
    "ver_usuarios",
    "crear_usuario",
    "modificar_usuario",
    "eliminar_usuario",
    "cambiar_contrasena_usuario",
    // Permisos
    "ver_permisos",
    "asignar_permiso_a_usuario",
    "quitar_permiso_a_usuario",
    // Proveedores
    "ver_proveedor",
    "crear_proveedor",
    "modificar_proveedor",
    "eliminar_proveedor",
    // Clientes
    "ver_clientes",
    "crear_cliente",
    "modificar_cliente",
    "eliminar_cliente",
    // Categorias
    "ver_categorias",
    "crear_categorias",
    "modificar_categorias",
    "eliminar_categorias",
    // Sub Categorias
    "ver_sub_categorias",
    "crear_sub_categorias",
    "modificar_sub_categorias",
    "eliminar_sub_categorias",
    // Articulos
    "ver_articulos",
    "crear_articulos",
    "modificar_articulos",
    "eliminar_articulos",
    // Stock
    "ver_stock",
    "crear_stock",
    "modificar_stock",
    "eliminar_stock",
    // Ventas
    "ver_ventas",
    "crear_venta",
    "anular_venta",
    "vender_sin_stock",
    "generar_presupuesto",
    // Tipos de Venta
    "ver_tipos_venta",
    "crear_tipo_venta",
    "modificar_tipo_venta",
    "eliminar_tipo_venta",
    // Auditoria
    "ver_auditoria",
    // Cierres del día
    "ver_cierres",
    "crear_cierre",
    "reabrir_cierre",
    // Dólar
    "ver_dolar",
];

pub(crate) fn seed_permissions(conn: &Connection) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    for permission in PERMISSIONS {
        conn.execute(
            "INSERT OR IGNORE INTO permissions (permission, created) VALUES (?1, ?2)",
            rusqlite::params![permission, now],
        )?;
    }

    Ok(())
}
