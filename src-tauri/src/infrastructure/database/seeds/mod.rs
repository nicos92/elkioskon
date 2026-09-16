mod admin;
mod cliente;
mod demo_data;
mod nocturno_config;
mod permissions;
mod proveedor;
mod tipos_venta;

pub(crate) use admin::seed_admin_user;
pub(crate) use cliente::seed_cliente_defecto;
pub(crate) use demo_data::seed_demo_data;
pub(crate) use nocturno_config::seed_nocturno_config;
pub(crate) use permissions::seed_permissions;
pub(crate) use proveedor::seed_proveedor_defecto;
pub(crate) use tipos_venta::seed_tipos_venta;
