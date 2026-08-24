use rusqlite::Connection;

pub(crate) fn seed_proveedor_defecto(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO proveedores (proveedor, nombre)
         SELECT 'Sin Proveedor', 'Sin Proveedor'
         WHERE NOT EXISTS (
             SELECT 1 FROM proveedores WHERE proveedor = 'Sin Proveedor'
         )",
        [],
    )?;

    Ok(())
}
