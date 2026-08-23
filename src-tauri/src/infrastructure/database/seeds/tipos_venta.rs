use rusqlite::Connection;

const TIPOS_VENTA: &[(&str, Option<&str>)] = &[
    ("Efectivo", None),
    ("Tarjeta Crédito", None),
    ("Tarjeta Débito", None),
    ("Transferencia", None),
    ("QR", None),
];

pub(crate) fn seed_tipos_venta(conn: &Connection) -> Result<(), rusqlite::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    for (nombre, hacia_donde) in TIPOS_VENTA {
        conn.execute(
            "INSERT OR IGNORE INTO tipos_venta (nombre, hacia_donde, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![nombre, hacia_donde, now],
        )?;
    }

    Ok(())
}
