use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionCode {
    ViewUsers,
    CreateUser,
    UpdateUser,
    DeleteUser,
    ChangeUserPassword,
    ViewPermissions,
    AssignPermission,
    RemovePermission,
    ViewProveedores,
    CreateProveedor,
    UpdateProveedor,
    DeleteProveedor,
    ViewClientes,
    CreateCliente,
    UpdateCliente,
    DeleteCliente,
    ViewCategorias,
    CreateCategoria,
    UpdateCategoria,
    DeleteCategoria,
    ViewSubCategorias,
    CreateSubCategoria,
    UpdateSubCategoria,
    DeleteSubCategoria,
    ViewArticulos,
    CreateArticulo,
    UpdateArticulo,
    DeleteArticulo,
    ViewStock,
    CreateStock,
    UpdateStock,
    DeleteStock,
    ViewVentas,
    CreateVenta,
    AnularVenta,
    VenderSinStock,
    GenerarPresupuesto,
    ViewTiposVenta,
    CreateTipoVenta,
    UpdateTipoVenta,
    DeleteTipoVenta,
    ViewAuditoria,
    ViewCierres,
    CreateCierre,
    ReopenCierre,
    ViewDolar,
    ConfigurarRecargoNocturno,
}

impl PermissionCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionCode::ViewUsers => "ver_usuarios",
            PermissionCode::CreateUser => "crear_usuario",
            PermissionCode::UpdateUser => "modificar_usuario",
            PermissionCode::DeleteUser => "eliminar_usuario",
            PermissionCode::ChangeUserPassword => "cambiar_contrasena_usuario",
            PermissionCode::ViewPermissions => "ver_permisos",
            PermissionCode::AssignPermission => "asignar_permiso_a_usuario",
            PermissionCode::RemovePermission => "quitar_permiso_a_usuario",
            PermissionCode::ViewProveedores => "ver_proveedor",
            PermissionCode::CreateProveedor => "crear_proveedor",
            PermissionCode::UpdateProveedor => "modificar_proveedor",
            PermissionCode::DeleteProveedor => "eliminar_proveedor",
            PermissionCode::ViewClientes => "ver_clientes",
            PermissionCode::CreateCliente => "crear_cliente",
            PermissionCode::UpdateCliente => "modificar_cliente",
            PermissionCode::DeleteCliente => "eliminar_cliente",
            PermissionCode::ViewCategorias => "ver_categorias",
            PermissionCode::CreateCategoria => "crear_categorias",
            PermissionCode::UpdateCategoria => "modificar_categorias",
            PermissionCode::DeleteCategoria => "eliminar_categorias",
            PermissionCode::ViewSubCategorias => "ver_sub_categorias",
            PermissionCode::CreateSubCategoria => "crear_sub_categorias",
            PermissionCode::UpdateSubCategoria => "modificar_sub_categorias",
            PermissionCode::DeleteSubCategoria => "eliminar_sub_categorias",
            PermissionCode::ViewArticulos => "ver_articulos",
            PermissionCode::CreateArticulo => "crear_articulos",
            PermissionCode::UpdateArticulo => "modificar_articulos",
            PermissionCode::DeleteArticulo => "eliminar_articulos",
            PermissionCode::ViewStock => "ver_stock",
            PermissionCode::CreateStock => "crear_stock",
            PermissionCode::UpdateStock => "modificar_stock",
            PermissionCode::DeleteStock => "eliminar_stock",
            PermissionCode::ViewVentas => "ver_ventas",
            PermissionCode::CreateVenta => "crear_venta",
            PermissionCode::AnularVenta => "anular_venta",
            PermissionCode::VenderSinStock => "vender_sin_stock",
            PermissionCode::GenerarPresupuesto => "generar_presupuesto",
            PermissionCode::ViewTiposVenta => "ver_tipos_venta",
            PermissionCode::CreateTipoVenta => "crear_tipo_venta",
            PermissionCode::UpdateTipoVenta => "modificar_tipo_venta",
            PermissionCode::DeleteTipoVenta => "eliminar_tipo_venta",
            PermissionCode::ViewAuditoria => "ver_auditoria",
            PermissionCode::ViewCierres => "ver_cierres",
            PermissionCode::CreateCierre => "crear_cierre",
            PermissionCode::ReopenCierre => "reabrir_cierre",
            PermissionCode::ViewDolar => "ver_dolar",
            PermissionCode::ConfigurarRecargoNocturno => "configurar_recargo_nocturno",
        }
    }

    pub fn all() -> Vec<PermissionCode> {
        vec![
            PermissionCode::ViewUsers,
            PermissionCode::CreateUser,
            PermissionCode::UpdateUser,
            PermissionCode::DeleteUser,
            PermissionCode::ChangeUserPassword,
            PermissionCode::ViewPermissions,
            PermissionCode::AssignPermission,
            PermissionCode::RemovePermission,
            PermissionCode::ViewProveedores,
            PermissionCode::CreateProveedor,
            PermissionCode::UpdateProveedor,
            PermissionCode::DeleteProveedor,
            PermissionCode::ViewClientes,
            PermissionCode::CreateCliente,
            PermissionCode::UpdateCliente,
            PermissionCode::DeleteCliente,
            PermissionCode::ViewCategorias,
            PermissionCode::CreateCategoria,
            PermissionCode::UpdateCategoria,
            PermissionCode::DeleteCategoria,
            PermissionCode::ViewSubCategorias,
            PermissionCode::CreateSubCategoria,
            PermissionCode::UpdateSubCategoria,
            PermissionCode::DeleteSubCategoria,
            PermissionCode::ViewArticulos,
            PermissionCode::CreateArticulo,
            PermissionCode::UpdateArticulo,
            PermissionCode::DeleteArticulo,
            PermissionCode::ViewStock,
            PermissionCode::CreateStock,
            PermissionCode::UpdateStock,
            PermissionCode::DeleteStock,
            PermissionCode::ViewVentas,
            PermissionCode::CreateVenta,
            PermissionCode::AnularVenta,
            PermissionCode::VenderSinStock,
            PermissionCode::GenerarPresupuesto,
            PermissionCode::ViewTiposVenta,
            PermissionCode::CreateTipoVenta,
            PermissionCode::UpdateTipoVenta,
            PermissionCode::DeleteTipoVenta,
            PermissionCode::ViewAuditoria,
            PermissionCode::ViewCierres,
            PermissionCode::CreateCierre,
            PermissionCode::ReopenCierre,
            PermissionCode::ViewDolar,
            PermissionCode::ConfigurarRecargoNocturno,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn as_str_returns_non_empty_snake_case_for_all_variants() {
        for code in PermissionCode::all() {
            let s = code.as_str();
            assert!(!s.is_empty(), "empty str for {:?}", code);
            assert!(!s.contains(' '), "space in '{}'", s);
            assert_eq!(s, s.to_lowercase(), "not lowercase: '{}'", s);
            assert!(!s.starts_with('_'), "leading underscore: '{}'", s);
            assert!(!s.ends_with('_'), "trailing underscore: '{}'", s);
        }
    }

    #[test]
    fn all_contains_no_duplicates() {
        let strings: Vec<String> = PermissionCode::all()
            .iter()
            .map(|c| c.as_str().to_string())
            .collect();
        let set: HashSet<&String> = strings.iter().collect();
        assert_eq!(strings.len(), set.len());
    }

    #[test]
    fn all_covers_seeded_permissions() {
        let expected: Vec<&str> = vec![
            "ver_usuarios",
            "crear_usuario",
            "modificar_usuario",
            "eliminar_usuario",
            "cambiar_contrasena_usuario",
            "ver_permisos",
            "asignar_permiso_a_usuario",
            "quitar_permiso_a_usuario",
            "ver_proveedor",
            "crear_proveedor",
            "modificar_proveedor",
            "eliminar_proveedor",
            "ver_clientes",
            "crear_cliente",
            "modificar_cliente",
            "eliminar_cliente",
            "ver_categorias",
            "crear_categorias",
            "modificar_categorias",
            "eliminar_categorias",
            "ver_sub_categorias",
            "crear_sub_categorias",
            "modificar_sub_categorias",
            "eliminar_sub_categorias",
            "ver_articulos",
            "crear_articulos",
            "modificar_articulos",
            "eliminar_articulos",
            "ver_stock",
            "crear_stock",
            "modificar_stock",
            "eliminar_stock",
            "ver_ventas",
            "crear_venta",
            "anular_venta",
            "vender_sin_stock",
            "generar_presupuesto",
            "ver_tipos_venta",
            "crear_tipo_venta",
            "modificar_tipo_venta",
            "eliminar_tipo_venta",
            "ver_auditoria",
            "ver_cierres",
            "crear_cierre",
            "reabrir_cierre",
            "ver_dolar",
            "configurar_recargo_nocturno",
        ];

        let mut actual: Vec<String> = PermissionCode::all()
            .iter()
            .map(|c| c.as_str().to_string())
            .collect();
        let mut expected: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        actual.sort_unstable();
        expected.sort_unstable();
        assert_eq!(actual, expected);
    }
}
