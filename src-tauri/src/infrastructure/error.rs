use rusqlite::ffi::ErrorCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Foreign key constraint violation")]
    ForeignKeyConstraint,

    #[error("Duplicate value violates uniqueness constraint")]
    DuplicateValue,

    #[error("User not found")]
    UserNotFound,

    #[error("Permission not found")]
    PermissionNotFound,

    #[error("Username already exists")]
    UsernameExists,

    #[error("Permission already exists")]
    PermissionExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User is inactive")]
    UserInactive,

    #[error("Hashing error: {0}")]
    Hashing(String),

    #[error("Permission already assigned to user")]
    PermissionAlreadyAssigned,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Proveedor not found")]
    ProveedorNotFound,

    #[error("CUIT already exists")]
    DuplicateCuit,

    #[error("Categoria not found")]
    CategoriaNotFound,

    #[error("Categoria already exists")]
    CategoriaExists,

    #[error("Categoria has sub-categorias")]
    CategoriaHasSubCategorias,

    #[error("Sub categoria not found")]
    SubCategoriaNotFound,

    #[error("Sub categoria already exists")]
    SubCategoriaExists,

    #[error("Sub categoria has articulos")]
    SubCategoriaHasArticulos,

    #[error("Articulo not found")]
    ArticuloNotFound,

    #[error("Codigo de articulo already exists")]
    CodArticuloExists,

    #[error("Stock not found")]
    StockNotFound,

    #[error("Stock already exists for this articulo")]
    StockExistsForArticulo,

    #[error("Proveedor has articulos")]
    ProveedorHasArticulos,

    #[error("Cliente not found")]
    ClienteNotFound,

    #[error("El cliente debe contar al menos con un dato de contacto")]
    ClienteSinDatosDeContacto,

    #[error("Cannot delete the default client")]
    NoSePuedeEliminarClienteDefecto,

    #[error("Default client not found")]
    ClienteDefectoNotFound,

    #[error("Cannot delete the current user")]
    CannotDeleteSelf,

    #[error("Cannot delete the admin user")]
    CannotDeleteAdmin,

    #[error("Password cannot be empty")]
    EmptyPassword,

    #[error("Venta not found")]
    VentaNotFound,

    #[error("Venta already anulada")]
    VentaAlreadyAnulada,

    #[error("Presupuesto not found")]
    PresupuestoNotFound,

    #[error("Presupuesto estado inválido")]
    PresupuestoEstadoInvalido,

    #[error("Insufficient stock")]
    InsufficientStock,

    #[error("Articulo has no stock")]
    ArticuloWithoutStock,

    #[error("Descuento inválido")]
    DescuentoInvalido,

    #[error("Tipo de venta not found")]
    TipoVentaNotFound,

    #[error("Tipo de venta already exists")]
    TipoVentaExists,

    #[error("Tipo de venta in use")]
    TipoVentaInUse,

    #[error("Stock has ventas")]
    StockHasVentas,

    #[error("Tipo de venta nombre inválido")]
    TipoVentaNombreInvalido,

    #[error("Cierre ya existe")]
    CierreYaExiste,

    #[error("Cierre not found")]
    CierreNotFound,

    #[error("Cierre sin ventas")]
    CierreSinVentas,

    #[error("Cierre fecha futura")]
    CierreFechaFutura,

    #[error("Día cerrado")]
    DiaCerrado,

    #[error("Día cerrado, no se puede anular la venta")]
    DiaCerradoAnulacion,

    #[error("No se pudo obtener la cotización del dólar: {0}")]
    DollarFetchError(String),

    #[error("Cotización del dólar not found")]
    DollarQuoteNotFound,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Porcentaje inválido para actualización masiva")]
    BulkUpdateInvalidPorcentaje,

    #[error("No se encontraron artículos con los filtros seleccionados")]
    BulkUpdateNoMatches,

    #[error("No se encontró la operación de actualización de costos")]
    CostUpdateNotFound,

    #[error("La operación ya fue deshecha")]
    CostUpdateAlreadyUndone,

    #[error("No se puede deshacer: {0} artículo(s) fueron modificados después de la operación")]
    CostUpdateModifiedAfter(i64),
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::SqliteFailure(ffi_error, msg) => {
                let extended_code = ffi_error.extended_code;

                if ffi_error.code == ErrorCode::ConstraintViolation {
                    match extended_code {
                        787 => AppError::ForeignKeyConstraint,
                        2067 => AppError::DuplicateValue,
                        _ => AppError::Database(msg.unwrap_or_else(|| ffi_error.to_string())),
                    }
                } else {
                    AppError::Database(msg.unwrap_or_else(|| ffi_error.to_string()))
                }
            }
            other => AppError::Database(other.to_string()),
        }
    }
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "database_error",
            AppError::ForeignKeyConstraint => "foreign_key_constraint",
            AppError::DuplicateValue => "duplicate_value",
            AppError::UserNotFound => "user_not_found",
            AppError::PermissionNotFound => "permission_not_found",
            AppError::UsernameExists => "username_exists",
            AppError::PermissionExists => "permission_exists",
            AppError::InvalidCredentials => "invalid_credentials",
            AppError::UserInactive => "user_inactive",
            AppError::Hashing(_) => "hashing_error",
            AppError::PermissionAlreadyAssigned => "permission_already_assigned",
            AppError::PermissionDenied => "permission_denied",
            AppError::ProveedorNotFound => "proveedor_not_found",
            AppError::DuplicateCuit => "duplicate_cuit",
            AppError::CategoriaNotFound => "categoria_not_found",
            AppError::CategoriaExists => "categoria_exists",
            AppError::CategoriaHasSubCategorias => "categoria_has_sub_categorias",
            AppError::SubCategoriaNotFound => "sub_categoria_not_found",
            AppError::SubCategoriaExists => "sub_categoria_exists",
            AppError::SubCategoriaHasArticulos => "sub_categoria_has_articulos",
            AppError::ArticuloNotFound => "articulo_not_found",
            AppError::CodArticuloExists => "cod_articulo_exists",
            AppError::StockNotFound => "stock_not_found",
            AppError::StockExistsForArticulo => "stock_exists_for_articulo",
            AppError::ProveedorHasArticulos => "proveedor_has_articulos",
            AppError::ClienteNotFound => "cliente_not_found",
            AppError::ClienteSinDatosDeContacto => "cliente_sin_datos_de_contacto",
            AppError::NoSePuedeEliminarClienteDefecto => "no_se_puede_eliminar_cliente_defecto",
            AppError::ClienteDefectoNotFound => "cliente_defecto_not_found",
            AppError::CannotDeleteSelf => "cannot_delete_self",
            AppError::CannotDeleteAdmin => "cannot_delete_admin",
            AppError::EmptyPassword => "empty_password",
            AppError::VentaNotFound => "venta_not_found",
            AppError::VentaAlreadyAnulada => "venta_already_anulada",
            AppError::PresupuestoNotFound => "presupuesto_not_found",
            AppError::PresupuestoEstadoInvalido => "presupuesto_estado_invalido",
            AppError::InsufficientStock => "insufficient_stock",
            AppError::ArticuloWithoutStock => "articulo_without_stock",
            AppError::DescuentoInvalido => "descuento_invalido",
            AppError::TipoVentaNotFound => "tipo_venta_not_found",
            AppError::TipoVentaExists => "tipo_venta_exists",
            AppError::TipoVentaInUse => "tipo_venta_in_use",
            AppError::StockHasVentas => "stock_has_ventas",
            AppError::TipoVentaNombreInvalido => "tipo_venta_nombre_invalido",
            AppError::CierreYaExiste => "cierre_ya_existe",
            AppError::CierreNotFound => "cierre_not_found",
            AppError::CierreSinVentas => "cierre_sin_ventas",
            AppError::CierreFechaFutura => "cierre_fecha_futura",
            AppError::DiaCerrado => "dia_cerrado",
            AppError::DiaCerradoAnulacion => "dia_cerrado_anulacion",
            AppError::DollarFetchError(_) => "dollar_fetch_error",
            AppError::DollarQuoteNotFound => "dollar_quote_not_found",
            AppError::Internal(_) => "internal_error",
            AppError::BulkUpdateInvalidPorcentaje => "bulk_update_invalid_porcentaje",
            AppError::BulkUpdateNoMatches => "bulk_update_no_matches",
            AppError::CostUpdateNotFound => "cost_update_not_found",
            AppError::CostUpdateAlreadyUndone => "cost_update_already_undone",
            AppError::CostUpdateModifiedAfter(_) => "cost_update_modified_after",
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            AppError::Database(_) => {
                "Ocurrió un error en la base de datos. Intente nuevamente.".to_string()
            }
            AppError::ForeignKeyConstraint => {
                "No se puede eliminar porque otros registros hacen referencia a este elemento."
                    .to_string()
            }
            AppError::DuplicateValue => "Ya existe un registro con ese valor único.".to_string(),
            AppError::UserNotFound => "El usuario no existe.".to_string(),
            AppError::PermissionNotFound => "El permiso no existe.".to_string(),
            AppError::UsernameExists => "El nombre de usuario ya existe.".to_string(),
            AppError::PermissionExists => "El permiso ya existe.".to_string(),
            AppError::InvalidCredentials => "Usuario o contraseña incorrectos.".to_string(),
            AppError::UserInactive => "El usuario está inactivo.".to_string(),
            AppError::Hashing(_) => "Ocurrió un error al procesar la contraseña.".to_string(),
            AppError::PermissionAlreadyAssigned => {
                "El permiso ya está asignado a este usuario.".to_string()
            }
            AppError::PermissionDenied => {
                "No tiene permisos para realizar esta acción.".to_string()
            }
            AppError::ProveedorNotFound => "El proveedor no existe.".to_string(),
            AppError::DuplicateCuit => "El CUIT ya está registrado.".to_string(),
            AppError::CategoriaNotFound => "La categoría no existe.".to_string(),
            AppError::CategoriaExists => "La categoría ya existe.".to_string(),
            AppError::CategoriaHasSubCategorias => {
                "No se puede eliminar la categoría porque tiene sub categorías asociadas."
                    .to_string()
            }
            AppError::SubCategoriaNotFound => "La sub categoría no existe.".to_string(),
            AppError::SubCategoriaExists => "La sub categoría ya existe.".to_string(),
            AppError::SubCategoriaHasArticulos => {
                "No se puede eliminar la sub categoría porque tiene artículos asociados."
                    .to_string()
            }
            AppError::ArticuloNotFound => "El artículo no existe.".to_string(),
            AppError::CodArticuloExists => "El código de artículo ya está registrado.".to_string(),
            AppError::StockNotFound => "El stock no existe.".to_string(),
            AppError::StockExistsForArticulo => {
                "El artículo ya tiene stock registrado.".to_string()
            }
            AppError::ProveedorHasArticulos => {
                "No se puede eliminar el proveedor porque tiene artículos asociados.".to_string()
            }
            AppError::ClienteNotFound => "El cliente no existe.".to_string(),
            AppError::ClienteSinDatosDeContacto => {
                "El cliente debe contar al menos con un dato de contacto o identificación."
                    .to_string()
            }
            AppError::NoSePuedeEliminarClienteDefecto => {
                "No se puede eliminar el cliente 'Consumidor Final'.".to_string()
            }
            AppError::ClienteDefectoNotFound => {
                "El cliente por defecto no existe. Reinicie la base de datos.".to_string()
            }
            AppError::CannotDeleteSelf => "No se puede eliminar a sí mismo.".to_string(),
            AppError::CannotDeleteAdmin => {
                "No se puede eliminar el usuario administrador.".to_string()
            }
            AppError::EmptyPassword => "La contraseña no puede estar vacía.".to_string(),
            AppError::VentaNotFound => "La venta no existe.".to_string(),
            AppError::VentaAlreadyAnulada => "La venta ya fue anulada.".to_string(),
            AppError::PresupuestoNotFound => "El presupuesto no existe.".to_string(),
            AppError::PresupuestoEstadoInvalido => {
                "El presupuesto no está en un estado que permita esa operación.".to_string()
            }
            AppError::InsufficientStock => {
                "Stock insuficiente para uno de los artículos.".to_string()
            }
            AppError::ArticuloWithoutStock => {
                "Uno de los artículos no tiene stock registrado.".to_string()
            }
            AppError::DescuentoInvalido => "El descuento debe estar entre 0 y 100.".to_string(),
            AppError::TipoVentaNotFound => "El tipo de venta no existe.".to_string(),
            AppError::TipoVentaExists => "Ya existe un tipo de venta con ese nombre.".to_string(),
            AppError::TipoVentaInUse => {
                "No se puede eliminar el tipo de venta porque tiene ventas asociadas.".to_string()
            }
            AppError::StockHasVentas => {
                "No se puede eliminar el stock porque el artículo tiene ventas asociadas."
                    .to_string()
            }
            AppError::TipoVentaNombreInvalido => {
                "El nombre del tipo de venta no puede estar vacío.".to_string()
            }
            AppError::CierreYaExiste => "Ya existe un cierre para esa fecha.".to_string(),
            AppError::CierreNotFound => "El cierre no existe.".to_string(),
            AppError::CierreSinVentas => {
                "El día seleccionado no tiene ventas para cerrar.".to_string()
            }
            AppError::CierreFechaFutura => "No se puede cerrar una fecha futura.".to_string(),
            AppError::DiaCerrado => "Día cerrado, no se pueden ingresar más ventas.".to_string(),
            AppError::DiaCerradoAnulacion => {
                "El día está cerrado, no se puede anular la venta.".to_string()
            }
            AppError::DollarFetchError(_) => {
                "No se pudo obtener la cotización del dólar. Intente nuevamente más tarde."
                    .to_string()
            }
            AppError::DollarQuoteNotFound => "La cotización del dólar no existe.".to_string(),
            AppError::Internal(_) => "Ocurrió un error inesperado. Intente nuevamente.".to_string(),
            AppError::BulkUpdateInvalidPorcentaje => {
                "El porcentaje ingresado es inválido.".to_string()
            }
            AppError::BulkUpdateNoMatches => {
                "No se encontraron artículos con los filtros seleccionados.".to_string()
            }
            AppError::CostUpdateNotFound => {
                "No se encontró la operación de actualización de costos.".to_string()
            }
            AppError::CostUpdateAlreadyUndone => "La operación ya fue deshecha.".to_string(),
            AppError::CostUpdateModifiedAfter(count) => {
                format!(
                    "No se puede deshacer: {} artículo(s) fueron modificados después de la operación.",
                    count
                )
            }
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.user_message())?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_returns_expected_strings() {
        assert_eq!(AppError::UserNotFound.code(), "user_not_found");
        assert_eq!(AppError::PermissionDenied.code(), "permission_denied");
        assert_eq!(AppError::InsufficientStock.code(), "insufficient_stock");
        assert_eq!(AppError::CierreFechaFutura.code(), "cierre_fecha_futura");
        assert_eq!(
            AppError::ClienteSinDatosDeContacto.code(),
            "cliente_sin_datos_de_contacto"
        );
        assert_eq!(
            AppError::NoSePuedeEliminarClienteDefecto.code(),
            "no_se_puede_eliminar_cliente_defecto"
        );
        assert_eq!(
            AppError::DollarFetchError("x".to_string()).code(),
            "dollar_fetch_error"
        );
        assert_eq!(
            AppError::DollarQuoteNotFound.code(),
            "dollar_quote_not_found"
        );
        assert_eq!(
            AppError::PresupuestoEstadoInvalido.code(),
            "presupuesto_estado_invalido"
        );
        assert_eq!(AppError::Internal("x".to_string()).code(), "internal_error");
        assert_eq!(
            AppError::BulkUpdateInvalidPorcentaje.code(),
            "bulk_update_invalid_porcentaje"
        );
        assert_eq!(
            AppError::BulkUpdateNoMatches.code(),
            "bulk_update_no_matches"
        );
        assert_eq!(AppError::CostUpdateNotFound.code(), "cost_update_not_found");
        assert_eq!(
            AppError::CostUpdateAlreadyUndone.code(),
            "cost_update_already_undone"
        );
        assert_eq!(
            AppError::CostUpdateModifiedAfter(0).code(),
            "cost_update_modified_after"
        );
    }

    #[test]
    fn user_message_returns_spanish_messages() {
        assert_eq!(
            AppError::UserNotFound.user_message(),
            "El usuario no existe."
        );
        assert_eq!(
            AppError::DescuentoInvalido.user_message(),
            "El descuento debe estar entre 0 y 100."
        );
        assert_eq!(
            AppError::PresupuestoEstadoInvalido.user_message(),
            "El presupuesto no está en un estado que permita esa operación."
        );
        assert!(AppError::Database("boom".to_string())
            .user_message()
            .contains("base de datos"));
        assert!(AppError::Internal("boom".to_string())
            .user_message()
            .contains("inesperado"));
        assert!(AppError::BulkUpdateInvalidPorcentaje
            .user_message()
            .contains("inválido"));
        assert!(AppError::BulkUpdateNoMatches
            .user_message()
            .contains("No se encontraron"));
        assert!(AppError::CostUpdateNotFound
            .user_message()
            .contains("No se encontró"));
        assert!(AppError::CostUpdateAlreadyUndone
            .user_message()
            .contains("ya fue deshecha"));
        assert!(AppError::CostUpdateModifiedAfter(3)
            .user_message()
            .contains("3 artículo(s) fueron modificados"));
    }

    #[test]
    fn maps_unique_constraint_to_duplicate_value() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE t (a TEXT UNIQUE)")
            .unwrap();
        conn.execute("INSERT INTO t (a) VALUES ('x')", []).unwrap();
        let err = conn
            .execute("INSERT INTO t (a) VALUES ('x')", [])
            .unwrap_err();
        let app_err: AppError = err.into();
        assert!(matches!(app_err, AppError::DuplicateValue), "{:?}", app_err);
    }

    #[test]
    fn maps_foreign_key_constraint_to_foreign_key_error() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             CREATE TABLE parent (id INTEGER PRIMARY KEY);
             CREATE TABLE child (pid INTEGER NOT NULL REFERENCES parent(id));",
        )
        .unwrap();
        let err = conn
            .execute("INSERT INTO child (pid) VALUES (999)", [])
            .unwrap_err();
        let app_err: AppError = err.into();
        assert!(
            matches!(app_err, AppError::ForeignKeyConstraint),
            "{:?}",
            app_err
        );
    }
}
