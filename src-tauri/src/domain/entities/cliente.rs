use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cliente {
    pub id: i64,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub telefono: Option<String>,
    pub email: Option<String>,
    pub direccion: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Cliente {
    pub fn new(
        nombre: Option<String>,
        apellido: Option<String>,
        telefono: Option<String>,
        email: Option<String>,
        direccion: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: 0,
            nombre,
            apellido,
            telefono,
            email,
            direccion,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn is_default(&self) -> bool {
        self.nombre.as_deref() == Some("Consumidor") && self.apellido.as_deref() == Some("Final")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_id_zero_and_timestamps() {
        let c = Cliente::new(
            Some("Juan".to_string()),
            Some("Pérez".to_string()),
            None,
            None,
            None,
        );
        assert_eq!(c.id, 0);
        assert_eq!(c.nombre.as_deref(), Some("Juan"));
        assert_eq!(c.apellido.as_deref(), Some("Pérez"));
        assert!(c.telefono.is_none());
        assert!(c.email.is_none());
        assert!(c.direccion.is_none());
        assert!(!c.created_at.is_empty());
        assert_eq!(c.created_at, c.updated_at);
    }

    #[test]
    fn is_default_matches_consumidor_final() {
        let c = Cliente::new(
            Some("Consumidor".to_string()),
            Some("Final".to_string()),
            None,
            None,
            None,
        );
        assert!(c.is_default());
    }

    #[test]
    fn is_default_false_for_other_clients() {
        let c = Cliente::new(
            Some("Consumidor".to_string()),
            Some("Anonimo".to_string()),
            None,
            None,
            None,
        );
        assert!(!c.is_default());

        let c2 = Cliente::new(None, Some("Final".to_string()), None, None, None);
        assert!(!c2.is_default());
    }

    #[test]
    fn json_round_trip() {
        let c = Cliente::new(
            Some("María".to_string()),
            None,
            Some("555-0000".to_string()),
            Some("m@example.com".to_string()),
            None,
        );
        let json = serde_json::to_string(&c).unwrap();
        let back: Cliente = serde_json::from_str(&json).unwrap();
        assert_eq!(back.nombre, c.nombre);
        assert_eq!(back.telefono, c.telefono);
        assert_eq!(back.email, c.email);
        assert_eq!(back.id, 0);
    }
}
