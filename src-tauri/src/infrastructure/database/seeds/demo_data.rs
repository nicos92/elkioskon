use rusqlite::Connection;

use crate::infrastructure::database::config::BCRYPT_COST;

const DEMO_USERS: &[(&str, &str)] = &[
    ("vendedor1", "vendedor1"),
    ("vendedor2", "vendedor2"),
    ("stock1", "stock1"),
    ("auditor1", "auditor1"),
    ("cajero1", "cajero1"),
    ("gerente1", "gerente1"),
];

const DEMO_USER_PERMISSIONS: &[(&str, &[&str])] = &[
    (
        "vendedor1",
        &[
            "ver_ventas",
            "crear_venta",
            "anular_venta",
            "generar_presupuesto",
            "vender_sin_stock",
            "ver_clientes",
            "crear_cliente",
            "ver_articulos",
            "ver_stock",
            "ver_tipos_venta",
        ],
    ),
    (
        "vendedor2",
        &[
            "ver_ventas",
            "crear_venta",
            "anular_venta",
            "generar_presupuesto",
            "vender_sin_stock",
            "ver_clientes",
            "crear_cliente",
            "ver_articulos",
            "ver_stock",
            "ver_tipos_venta",
        ],
    ),
    (
        "stock1",
        &[
            "ver_stock",
            "crear_stock",
            "modificar_stock",
            "ver_articulos",
            "crear_articulos",
            "modificar_articulos",
            "ver_proveedor",
            "crear_proveedor",
            "modificar_proveedor",
            "ver_categorias",
            "crear_categorias",
            "modificar_categorias",
            "ver_sub_categorias",
            "crear_sub_categorias",
            "modificar_sub_categorias",
        ],
    ),
    (
        "auditor1",
        &[
            "ver_auditoria",
            "ver_cierres",
            "ver_usuarios",
            "ver_permisos",
        ],
    ),
    (
        "cajero1",
        &[
            "ver_ventas",
            "crear_venta",
            "anular_venta",
            "generar_presupuesto",
            "ver_tipos_venta",
        ],
    ),
    (
        "gerente1",
        &[
            "ver_usuarios",
            "crear_usuario",
            "modificar_usuario",
            "cambiar_contrasena_usuario",
            "ver_permisos",
            "asignar_permiso_a_usuario",
            "quitar_permiso_a_usuario",
            "ver_proveedor",
            "crear_proveedor",
            "modificar_proveedor",
            "ver_clientes",
            "crear_cliente",
            "modificar_cliente",
            "ver_categorias",
            "crear_categorias",
            "modificar_categorias",
            "ver_sub_categorias",
            "crear_sub_categorias",
            "modificar_sub_categorias",
            "ver_articulos",
            "crear_articulos",
            "modificar_articulos",
            "ver_stock",
            "crear_stock",
            "modificar_stock",
            "ver_ventas",
            "crear_venta",
            "anular_venta",
            "vender_sin_stock",
            "generar_presupuesto",
            "ver_tipos_venta",
            "crear_tipo_venta",
            "modificar_tipo_venta",
            "ver_auditoria",
            "ver_cierres",
            "crear_cierre",
            "reabrir_cierre",
        ],
    ),
];

pub(crate) fn seed_demo_data(conn: &Connection) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    for (username, password) in DEMO_USERS {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?1)",
            rusqlite::params![username],
            |row| row.get(0),
        )?;
        if !exists {
            let hashed =
                bcrypt::hash(password, BCRYPT_COST).expect("Failed to hash demo user password");
            conn.execute(
                "INSERT INTO users (username, password, active, created_at, modified_at) VALUES (?1, ?2, 1, ?3, ?3)",
                rusqlite::params![username, hashed, now],
            )?;
        }
    }

    for (username, perms) in DEMO_USER_PERMISSIONS {
        let user_id: i64 = conn.query_row(
            "SELECT id FROM users WHERE username = ?1",
            rusqlite::params![username],
            |row| row.get(0),
        )?;
        for perm in *perms {
            conn.execute(
                "INSERT OR IGNORE INTO user_permissions (user_id, permission_id, assigned_at)
                 SELECT ?1, id, ?2 FROM permissions WHERE permission = ?3",
                rusqlite::params![user_id, now, perm],
            )?;
        }
    }

    Ok(())
}
