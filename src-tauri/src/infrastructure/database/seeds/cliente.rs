use rusqlite::Connection;

pub(crate) fn seed_cliente_defecto(conn: &Connection) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO clientes (nombre, apellido, telefono, email, direccion, created_at, updated_at)
         SELECT 'Consumidor', 'Final', NULL, NULL, NULL, ?1, ?1
         WHERE NOT EXISTS (
             SELECT 1 FROM clientes WHERE nombre = 'Consumidor' AND apellido = 'Final'
         )",
        rusqlite::params![now],
    )?;

    Ok(())
}
