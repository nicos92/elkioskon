use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub id: i64,
    pub id_articulo: i64,
    pub cantidad: f64,
    pub costo: f64,
    pub ganancia: f64,
    pub ganancia_diurna: f64,
    pub ganancia_nocturna: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl Stock {
    pub fn new(id_articulo: i64, cantidad: f64, costo: f64, ganancia: f64) -> Self {
        Self {
            id: 0,
            id_articulo,
            cantidad,
            costo,
            ganancia,
            ganancia_diurna: 0.0,
            ganancia_nocturna: 0.0,
            updated_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_fields() {
        let s = Stock::new(12, 50.5, 100.0, 30.0);
        assert_eq!(s.id, 0);
        assert_eq!(s.id_articulo, 12);
        assert_eq!(s.cantidad, 50.5);
        assert_eq!(s.costo, 100.0);
        assert_eq!(s.ganancia, 30.0);
        assert_eq!(s.ganancia_diurna, 0.0);
        assert_eq!(s.ganancia_nocturna, 0.0);
        assert!(s.updated_at.is_none());
    }

    #[test]
    fn new_accepts_zero_and_negative_cantidad() {
        let s = Stock::new(12, 0.0, 0.0, 0.0);
        assert_eq!(s.cantidad, 0.0);
        assert_eq!(s.costo, 0.0);
        assert_eq!(s.ganancia, 0.0);
        assert_eq!(s.ganancia_diurna, 0.0);
        assert_eq!(s.ganancia_nocturna, 0.0);
        assert!(s.updated_at.is_none());
    }

    #[test]
    fn json_round_trip() {
        let s = Stock {
            id: 12,
            id_articulo: 5,
            cantidad: 50.5,
            costo: 100.0,
            ganancia: 30.0,
            ganancia_diurna: 10.0,
            ganancia_nocturna: 40.0,
            updated_at: Some("2026-01-01T00:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Stock = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id_articulo, 5);
        assert_eq!(back.cantidad, 50.5);
        assert_eq!(back.costo, 100.0);
        assert_eq!(back.ganancia, 30.0);
        assert_eq!(back.ganancia_diurna, 10.0);
        assert_eq!(back.ganancia_nocturna, 40.0);
        assert_eq!(back.id, 12);
        assert_eq!(back.updated_at.as_deref(), Some("2026-01-01T00:00:00Z"));
    }

    #[test]
    fn json_omits_updated_at_when_none() {
        let s = Stock::new(1, 10.0, 100.0, 25.0);
        let json = serde_json::to_string(&s).unwrap();
        assert!(!json.contains("updated_at"));
    }
}
