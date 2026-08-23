use rusqlite::Connection;

#[cfg(test)]
pub(crate) const TABLES: &[&str] = &[
    "cost_update_items",
    "cost_update_operations",
    "detalle_presupuestos",
    "presupuestos",
    "cierre_tipos",
    "cierres",
    "venta_detalle",
    "ventas",
    "audit_logs",
    "stock",
    "articulos",
    "sub_categorias",
    "categorias",
    "proveedores",
    "clientes",
    "user_permissions",
    "users",
    "permissions",
    "tipos_venta",
    "dollar_quotes",
];

const SCHEMA_SQL: &str = "
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT NOT NULL UNIQUE,
        password TEXT NOT NULL,
        active INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL,
        modified_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS permissions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        permission TEXT NOT NULL UNIQUE,
        created TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS user_permissions (
        user_id INTEGER NOT NULL,
        permission_id INTEGER NOT NULL,
        assigned_at TEXT NOT NULL,
        PRIMARY KEY (user_id, permission_id),
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
        FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS proveedores (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        cuit TEXT UNIQUE,
        proveedor TEXT NOT NULL,
        nombre TEXT NOT NULL,
        tel TEXT,
        email TEXT,
        observacion TEXT
    );

    CREATE TABLE IF NOT EXISTS clientes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        nombre TEXT,
        apellido TEXT,
        telefono TEXT,
        email TEXT,
        direccion TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS categorias (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        categoria TEXT NOT NULL UNIQUE
    );

    CREATE TABLE IF NOT EXISTS sub_categorias (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        sub_categoria TEXT NOT NULL UNIQUE,
        id_categoria INTEGER NOT NULL,
        FOREIGN KEY (id_categoria) REFERENCES categorias(id)
    );

    CREATE TABLE IF NOT EXISTS articulos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        articulo TEXT NOT NULL UNIQUE,
        cod_articulo TEXT NOT NULL UNIQUE,
        id_sub_categoria INTEGER NOT NULL,
        id_proveedor INTEGER NOT NULL,
        FOREIGN KEY (id_sub_categoria) REFERENCES sub_categorias(id),
        FOREIGN KEY (id_proveedor) REFERENCES proveedores(id)
    );

    CREATE TABLE IF NOT EXISTS stock (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        id_articulo INTEGER NOT NULL,
        cantidad REAL NOT NULL,
        costo REAL NOT NULL,
        ganancia REAL NOT NULL,
        FOREIGN KEY (id_articulo) REFERENCES articulos(id)
    );

    CREATE TABLE IF NOT EXISTS audit_logs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        username TEXT NOT NULL,
        screen TEXT NOT NULL,
        action TEXT NOT NULL,
        detail TEXT,
        created_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);
    CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
    CREATE INDEX IF NOT EXISTS idx_audit_logs_screen_action ON audit_logs(screen, action);

    CREATE TABLE IF NOT EXISTS tipos_venta (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        nombre TEXT NOT NULL UNIQUE,
        hacia_donde TEXT,
        created_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS ventas (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        fecha TEXT NOT NULL,
        total REAL NOT NULL,
        descuento REAL NOT NULL DEFAULT 0,
        anulada INTEGER NOT NULL DEFAULT 0,
        observacion TEXT,
        id_tipo_venta INTEGER REFERENCES tipos_venta(id),
        cliente_id INTEGER NOT NULL REFERENCES clientes(id),
        created_at TEXT NOT NULL,
        FOREIGN KEY (user_id) REFERENCES users(id)
    );

    CREATE TABLE IF NOT EXISTS venta_detalle (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        id_venta INTEGER NOT NULL,
        id_articulo INTEGER NOT NULL,
        cantidad REAL NOT NULL,
        costo_unitario REAL NOT NULL,
        precio_unitario REAL NOT NULL,
        subtotal REAL NOT NULL,
        FOREIGN KEY (id_venta) REFERENCES ventas(id) ON DELETE CASCADE,
        FOREIGN KEY (id_articulo) REFERENCES articulos(id)
    );

    CREATE INDEX IF NOT EXISTS idx_venta_detalle_id_venta ON venta_detalle(id_venta);

    CREATE TABLE IF NOT EXISTS presupuestos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        fecha TEXT NOT NULL,
        total REAL NOT NULL,
        descuento REAL NOT NULL DEFAULT 0,
        estado TEXT NOT NULL DEFAULT 'pendiente'
            CHECK (estado IN ('pendiente','aprobado','vencido','convertido','anulado')),
        fecha_vencimiento TEXT,
        observacion TEXT,
        cliente_id INTEGER REFERENCES clientes(id),
        created_at TEXT NOT NULL,
        FOREIGN KEY (user_id) REFERENCES users(id)
    );

    CREATE TABLE IF NOT EXISTS detalle_presupuestos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        id_presupuesto INTEGER NOT NULL,
        id_articulo INTEGER NOT NULL,
        cantidad REAL NOT NULL,
        costo_unitario REAL NOT NULL,
        precio_unitario REAL NOT NULL,
        subtotal REAL NOT NULL,
        FOREIGN KEY (id_presupuesto) REFERENCES presupuestos(id) ON DELETE CASCADE,
        FOREIGN KEY (id_articulo) REFERENCES articulos(id)
    );

    CREATE INDEX IF NOT EXISTS idx_detalle_presupuestos_id_presupuesto ON detalle_presupuestos(id_presupuesto);
    CREATE INDEX IF NOT EXISTS idx_presupuestos_estado ON presupuestos(estado);

    CREATE TABLE IF NOT EXISTS cierres (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        fecha TEXT NOT NULL UNIQUE,
        dia INTEGER NOT NULL,
        mes INTEGER NOT NULL,
        anio INTEGER NOT NULL,
        total_costo REAL NOT NULL,
        total_ganancia REAL NOT NULL,
        total_venta REAL NOT NULL,
        created_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS cierre_tipos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        id_cierre INTEGER NOT NULL,
        id_tipo_venta INTEGER NOT NULL,
        total REAL NOT NULL,
        FOREIGN KEY (id_cierre) REFERENCES cierres(id) ON DELETE CASCADE,
        FOREIGN KEY (id_tipo_venta) REFERENCES tipos_venta(id)
    );

    CREATE INDEX IF NOT EXISTS idx_cierre_tipos_id_cierre ON cierre_tipos(id_cierre);

    DROP TABLE IF EXISTS dollar_rates;

    CREATE TABLE IF NOT EXISTS dollar_quotes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        official_buy REAL NOT NULL,
        official_sell REAL NOT NULL,
        blue_buy REAL NOT NULL,
        blue_sell REAL NOT NULL,
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_dollar_quotes_timestamp ON dollar_quotes(timestamp, id);

    CREATE TABLE IF NOT EXISTS cost_update_operations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        porcentaje REAL NOT NULL,
        filtro_categoria INTEGER,
        filtro_sub_categoria INTEGER,
        filtro_proveedor INTEGER,
        affected_count INTEGER NOT NULL DEFAULT 0,
        estado TEXT NOT NULL DEFAULT 'aplicada'
            CHECK (estado IN ('aplicada', 'deshecha')),
        created_at TEXT NOT NULL,
        undone_at TEXT,
        FOREIGN KEY (user_id) REFERENCES users(id),
        FOREIGN KEY (filtro_categoria) REFERENCES categorias(id),
        FOREIGN KEY (filtro_sub_categoria) REFERENCES sub_categorias(id),
        FOREIGN KEY (filtro_proveedor) REFERENCES proveedores(id)
    );

    CREATE TABLE IF NOT EXISTS cost_update_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        operation_id INTEGER NOT NULL,
        id_stock INTEGER NOT NULL,
        costo_anterior REAL NOT NULL,
        costo_nuevo REAL NOT NULL,
        FOREIGN KEY (operation_id) REFERENCES cost_update_operations(id) ON DELETE CASCADE,
        FOREIGN KEY (id_stock) REFERENCES stock(id)
    );

    CREATE INDEX IF NOT EXISTS idx_cuo_estado ON cost_update_operations(estado);
    CREATE INDEX IF NOT EXISTS idx_cuo_created_at ON cost_update_operations(created_at);
    CREATE INDEX IF NOT EXISTS idx_cui_operation_id ON cost_update_items(operation_id);
";

pub(crate) fn apply_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(SCHEMA_SQL)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_const_matches_create_statements_in_schema_sql() {
        const MARKER: &str = "CREATE TABLE IF NOT EXISTS ";

        let mut ddl_tables: Vec<String> = Vec::new();
        let mut rest = SCHEMA_SQL;
        while let Some(start) = rest.find(MARKER) {
            let after = &rest[start + MARKER.len()..];
            let name: String = after
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            assert!(!name.is_empty(), "table name not found after marker");
            ddl_tables.push(name);
            rest = after;
        }

        let mut declared_tables: Vec<String> = TABLES.iter().map(|t| t.to_string()).collect();
        ddl_tables.sort();
        declared_tables.sort();

        assert_eq!(ddl_tables.len(), declared_tables.len());
        assert_eq!(ddl_tables, declared_tables);
    }
}
