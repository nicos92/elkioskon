use rusqlite::Connection;

use crate::infrastructure::database::config::BCRYPT_COST;

const DEMO_CATEGORIAS: &[&str] = &[
    "Cables",
    "Luminarias",
    "Interruptores",
    "Tomacorrientes",
    "Tableros eléctricos",
    "Disyuntores",
    "Accesorios de instalación",
];

const DEMO_SUB_CATEGORIAS: &[(&str, &str)] = &[
    ("Cable unipolar 2.5mm", "Cables"),
    ("Cable unipolar 4mm", "Cables"),
    ("Cable mellizo 2x1.5mm", "Cables"),
    ("Cable mellizo 2x2.5mm", "Cables"),
    ("Panel LED 60x60", "Luminarias"),
    ("Foco LED 9W", "Luminarias"),
    ("Reflector LED 50W", "Luminarias"),
    ("Tubo LED 18W", "Luminarias"),
    ("Interruptor simple", "Interruptores"),
    ("Interruptor doble", "Interruptores"),
    ("Interruptor con tecla luminosa", "Interruptores"),
    ("Tomacorriente simple", "Tomacorrientes"),
    ("Tomacorriente doble", "Tomacorrientes"),
    ("Tomacorriente con puesta a tierra", "Tomacorrientes"),
    ("Tablero embutido 6 bocas", "Tableros eléctricos"),
    ("Tablero embutido 12 bocas", "Tableros eléctricos"),
    ("Tablero superficial 6 bocas", "Tableros eléctricos"),
    ("Disyuntor 25A", "Disyuntores"),
    ("Disyuntor 40A", "Disyuntores"),
    ("Disyuntor 63A", "Disyuntores"),
    ("Caja de luz embutida", "Accesorios de instalación"),
    ("Caja de luz superficial", "Accesorios de instalación"),
    ("Conector rápido", "Accesorios de instalación"),
    ("Ficha macho 10A", "Accesorios de instalación"),
    ("Ficha hembra 10A", "Accesorios de instalación"),
];

const DEMO_PROVEEDORES: &[(&str, &str, &str, &str, &str, &str)] = &[
    (
        "30-12345678-9",
        "ElectroSur",
        "Carlos Gómez",
        "1122334455",
        "ventas@electrosur.com",
        "Mayorista de cables eléctricos",
    ),
    (
        "30-98765432-1",
        "LuzMax",
        "María Pérez",
        "1199887766",
        "contacto@luzmax.com",
        "Proveedor de luminarias LED",
    ),
    (
        "30-45678901-2",
        "TecnoSwitch",
        "Juan López",
        "1144556677",
        "info@tecnoswitch.com",
        "Especialista en interruptores y tomas",
    ),
    (
        "30-65432109-8",
        "PowerLine",
        "Ana Torres",
        "1133445566",
        "ventas@powerline.com",
        "Distribuidor de tableros eléctricos",
    ),
    (
        "30-11223344-5",
        "SegurElec",
        "Roberto Díaz",
        "1177889900",
        "seguridad@segurelec.com",
        "Proveedor de disyuntores y protección",
    ),
    (
        "30-99887766-4",
        "InstalMax",
        "Laura Fernández",
        "1166778899",
        "instalaciones@instalmax.com",
        "Accesorios de instalación y cajas",
    ),
    (
        "30-22334455-6",
        "Iluminarte",
        "Sofía Martínez",
        "1155667788",
        "ventas@iluminarte.com",
        "Especialista en reflectores y tubos LED",
    ),
];

const DEMO_ARTICULOS: &[(&str, &str, &str, &str)] = &[
    (
        "Cable unipolar 2.5mm rojo",
        "CAB-25R",
        "Cable unipolar 2.5mm",
        "30-12345678-9",
    ),
    (
        "Cable unipolar 4mm azul",
        "CAB-40A",
        "Cable unipolar 4mm",
        "30-12345678-9",
    ),
    (
        "Cable mellizo 2x1.5mm negro",
        "CAB-M15N",
        "Cable mellizo 2x1.5mm",
        "30-12345678-9",
    ),
    (
        "Cable mellizo 2x2.5mm blanco",
        "CAB-M25B",
        "Cable mellizo 2x2.5mm",
        "30-12345678-9",
    ),
    (
        "Panel LED 60x60 blanco",
        "LED-6060B",
        "Panel LED 60x60",
        "30-98765432-1",
    ),
    (
        "Foco LED 9W cálido",
        "FOC-9WC",
        "Foco LED 9W",
        "30-98765432-1",
    ),
    (
        "Reflector LED 50W exterior",
        "REF-50E",
        "Reflector LED 50W",
        "30-98765432-1",
    ),
    (
        "Tubo LED 18W frío",
        "TUB-18F",
        "Tubo LED 18W",
        "30-98765432-1",
    ),
    (
        "Interruptor simple blanco",
        "INT-SB",
        "Interruptor simple",
        "30-45678901-2",
    ),
    (
        "Interruptor doble gris",
        "INT-DG",
        "Interruptor doble",
        "30-45678901-2",
    ),
    (
        "Interruptor con tecla luminosa",
        "INT-LUM",
        "Interruptor con tecla luminosa",
        "30-45678901-2",
    ),
    (
        "Tomacorriente simple blanco",
        "TOM-SB",
        "Tomacorriente simple",
        "30-45678901-2",
    ),
    (
        "Tomacorriente doble gris",
        "TOM-DG",
        "Tomacorriente doble",
        "30-45678901-2",
    ),
    (
        "Tomacorriente con puesta a tierra",
        "TOM-PT",
        "Tomacorriente con puesta a tierra",
        "30-45678901-2",
    ),
    (
        "Tablero embutido 6 bocas",
        "TAB-E6",
        "Tablero embutido 6 bocas",
        "30-65432109-8",
    ),
    (
        "Tablero embutido 12 bocas",
        "TAB-E12",
        "Tablero embutido 12 bocas",
        "30-65432109-8",
    ),
    (
        "Tablero superficial 6 bocas",
        "TAB-S6",
        "Tablero superficial 6 bocas",
        "30-65432109-8",
    ),
    ("Disyuntor 25A", "DIS-25A", "Disyuntor 25A", "30-11223344-5"),
    ("Disyuntor 40A", "DIS-40A", "Disyuntor 40A", "30-11223344-5"),
    ("Disyuntor 63A", "DIS-63A", "Disyuntor 63A", "30-11223344-5"),
    (
        "Caja de luz embutida",
        "CAJ-E",
        "Caja de luz embutida",
        "30-99887766-4",
    ),
    (
        "Caja de luz superficial",
        "CAJ-S",
        "Caja de luz superficial",
        "30-99887766-4",
    ),
    (
        "Conector rápido universal",
        "CON-RU",
        "Conector rápido",
        "30-99887766-4",
    ),
    (
        "Ficha macho 10A",
        "FIC-M10",
        "Ficha macho 10A",
        "30-99887766-4",
    ),
    (
        "Ficha hembra 10A",
        "FIC-H10",
        "Ficha hembra 10A",
        "30-99887766-4",
    ),
];

const DEMO_STOCK: &[(&str, f64, f64, f64)] = &[
    ("CAB-25R", 100.0, 120.0, 30.0),
    ("CAB-40A", 80.0, 200.0, 50.0),
    ("CAB-M15N", 60.0, 150.0, 40.0),
    ("CAB-M25B", 50.0, 220.0, 60.0),
    ("LED-6060B", 40.0, 1500.0, 400.0),
    ("FOC-9WC", 200.0, 250.0, 70.0),
    ("REF-50E", 30.0, 1800.0, 500.0),
    ("TUB-18F", 100.0, 300.0, 80.0),
    ("INT-SB", 150.0, 180.0, 50.0),
    ("INT-DG", 120.0, 220.0, 60.0),
    ("INT-LUM", 100.0, 250.0, 70.0),
    ("TOM-SB", 200.0, 160.0, 40.0),
    ("TOM-DG", 150.0, 190.0, 50.0),
    ("TOM-PT", 120.0, 210.0, 55.0),
    ("TAB-E6", 40.0, 1200.0, 300.0),
    ("TAB-E12", 25.0, 1800.0, 450.0),
    ("TAB-S6", 30.0, 1300.0, 350.0),
    ("DIS-25A", 60.0, 700.0, 180.0),
    ("DIS-40A", 50.0, 900.0, 220.0),
    ("DIS-63A", 40.0, 1200.0, 300.0),
    ("CAJ-E", 200.0, 80.0, 20.0),
    ("CAJ-S", 150.0, 90.0, 25.0),
    ("CON-RU", 300.0, 50.0, 15.0),
    ("FIC-M10", 250.0, 70.0, 20.0),
    ("FIC-H10", 250.0, 70.0, 20.0),
];

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

    for categoria in DEMO_CATEGORIAS {
        conn.execute(
            "INSERT OR IGNORE INTO categorias (categoria) VALUES (?1)",
            rusqlite::params![categoria],
        )?;
    }

    for (sub_categoria, categoria) in DEMO_SUB_CATEGORIAS {
        let id_categoria: i64 = conn.query_row(
            "SELECT id FROM categorias WHERE categoria = ?1",
            rusqlite::params![categoria],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO sub_categorias (sub_categoria, id_categoria) VALUES (?1, ?2)",
            rusqlite::params![sub_categoria, id_categoria],
        )?;
    }

    for (cuit, proveedor, nombre, tel, email, observacion) in DEMO_PROVEEDORES {
        conn.execute(
            "INSERT OR IGNORE INTO proveedores (cuit, proveedor, nombre, tel, email, observacion) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![cuit, proveedor, nombre, tel, email, observacion],
        )?;
    }

    for (articulo, cod_articulo, sub_categoria, proveedor_cuit) in DEMO_ARTICULOS {
        let id_sub_categoria: i64 = conn.query_row(
            "SELECT id FROM sub_categorias WHERE sub_categoria = ?1",
            rusqlite::params![sub_categoria],
            |row| row.get(0),
        )?;
        let id_proveedor: i64 = conn.query_row(
            "SELECT id FROM proveedores WHERE cuit = ?1",
            rusqlite::params![proveedor_cuit],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO articulos (articulo, cod_articulo, id_sub_categoria, id_proveedor) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![articulo, cod_articulo, id_sub_categoria, id_proveedor],
        )?;
    }

    for (cod_articulo, cantidad, costo, ganancia) in DEMO_STOCK {
        let id_articulo: i64 = conn.query_row(
            "SELECT id FROM articulos WHERE cod_articulo = ?1",
            rusqlite::params![cod_articulo],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT INTO stock (id_articulo, cantidad, costo, ganancia, updated_at)
             SELECT ?1, ?2, ?3, ?4, ?5
             WHERE NOT EXISTS (SELECT 1 FROM stock WHERE id_articulo = ?1)",
            rusqlite::params![id_articulo, cantidad, costo, ganancia, now],
        )?;
    }

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
