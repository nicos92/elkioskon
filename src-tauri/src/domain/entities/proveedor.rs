use serde::{Deserialize, Serialize};

pub const DEFAULT_PROVEEDOR_NOMBRE: &str = "Sin Proveedor";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proveedor {
    pub id: i64,
    pub cuit: Option<String>,
    pub proveedor: String,
    pub nombre: String,
    pub tel: Option<String>,
    pub email: Option<String>,
    pub observacion: Option<String>,
}

impl Proveedor {
    pub fn new(
        proveedor: String,
        nombre: String,
        cuit: Option<String>,
        tel: Option<String>,
        email: Option<String>,
        observacion: Option<String>,
    ) -> Self {
        Self {
            id: 0,
            proveedor,
            nombre,
            cuit,
            tel,
            email,
            observacion,
        }
    }

    pub fn is_default(&self) -> bool {
        self.proveedor == DEFAULT_PROVEEDOR_NOMBRE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_id_zero_and_required_fields() {
        let p = Proveedor::new(
            "DISTRIBUIDORA".to_string(),
            "Distribuidora SA".to_string(),
            None,
            None,
            None,
            None,
        );
        assert_eq!(p.id, 0);
        assert_eq!(p.proveedor, "DISTRIBUIDORA");
        assert_eq!(p.nombre, "Distribuidora SA");
        assert!(p.cuit.is_none());
        assert!(p.tel.is_none());
        assert!(p.email.is_none());
        assert!(p.observacion.is_none());
    }

    #[test]
    fn new_keeps_optional_fields() {
        let p = Proveedor::new(
            "P1".to_string(),
            "Nombre 1".to_string(),
            Some("20-12345678-9".to_string()),
            Some("555-1234".to_string()),
            Some("a@b.com".to_string()),
            Some("obs".to_string()),
        );
        assert_eq!(p.cuit.as_deref(), Some("20-12345678-9"));
        assert_eq!(p.tel.as_deref(), Some("555-1234"));
        assert_eq!(p.email.as_deref(), Some("a@b.com"));
        assert_eq!(p.observacion.as_deref(), Some("obs"));
    }

    #[test]
    fn json_round_trip() {
        let p = Proveedor::new(
            "P1".to_string(),
            "Nombre 1".to_string(),
            Some("20-12345678-9".to_string()),
            None,
            None,
            None,
        );
        let json = serde_json::to_string(&p).unwrap();
        let back: Proveedor = serde_json::from_str(&json).unwrap();
        assert_eq!(back.proveedor, p.proveedor);
        assert_eq!(back.cuit, p.cuit);
        assert_eq!(back.id, 0);
    }

    #[test]
    fn is_default_matches_sin_proveedor() {
        let p = Proveedor::new(
            DEFAULT_PROVEEDOR_NOMBRE.to_string(),
            DEFAULT_PROVEEDOR_NOMBRE.to_string(),
            None,
            None,
            None,
            None,
        );
        assert!(p.is_default());
    }

    #[test]
    fn is_default_false_for_other_proveedores() {
        let exact_case = Proveedor::new(
            "sin proveedor".to_string(),
            "sin proveedor".to_string(),
            None,
            None,
            None,
            None,
        );
        assert!(!exact_case.is_default());

        let real = Proveedor::new(
            "ElectroSur".to_string(),
            "ElectroSur SA".to_string(),
            None,
            None,
            None,
            None,
        );
        assert!(!real.is_default());
    }
}
