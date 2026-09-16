use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuditCambio {
    pub campo: &'static str,
    pub antes: String,
    pub valor: String,
}

impl AuditCambio {
    pub fn new(campo: &'static str, antes: impl ToString, valor: impl ToString) -> Self {
        Self {
            campo,
            antes: antes.to_string(),
            valor: valor.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditDetail {
    pub tipo: &'static str,
    pub descripcion: String,
    pub cambios: Vec<AuditCambio>,
}

impl AuditDetail {
    pub fn new(tipo: &'static str, descripcion: impl Into<String>) -> Self {
        Self {
            tipo,
            descripcion: descripcion.into(),
            cambios: Vec::new(),
        }
    }

    pub fn cambio(
        mut self,
        campo: &'static str,
        antes: impl ToString,
        valor: impl ToString,
    ) -> Self {
        self.cambios.push(AuditCambio::new(campo, antes, valor));
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serializar AuditDetail a JSON")
    }
}

pub fn opt_str(val: &Option<String>) -> String {
    val.clone().unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct Cambio {
        campo: String,
        antes: String,
        valor: String,
    }

    #[derive(serde::Deserialize)]
    struct Payload {
        tipo: String,
        descripcion: String,
        cambios: Vec<Cambio>,
    }

    #[test]
    fn to_json_persiste_campos_y_cambios() {
        let json = AuditDetail::new("recargo_nocturno", "Recargo nocturno")
            .cambio("activo", false, true)
            .cambio("porcentaje", 0.0, 12.5)
            .cambio("hora_inicio", "22:00", "23:00")
            .cambio("hora_fin", "06:00", "07:00")
            .to_json();

        let payload: Payload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload.tipo, "recargo_nocturno");
        assert_eq!(payload.descripcion, "Recargo nocturno");
        assert_eq!(payload.cambios.len(), 4);
        assert_eq!(payload.cambios[0].campo, "activo");
        assert_eq!(payload.cambios[0].antes, "false");
        assert_eq!(payload.cambios[0].valor, "true");
        assert_eq!(payload.cambios[2].campo, "hora_inicio");
        assert_eq!(payload.cambios[2].valor, "23:00");
    }

    #[test]
    fn to_json_genera_objeto_con_campos_esperados() {
        let json = AuditDetail::new("categoria", "Bebidas")
            .cambio("categoria", "Bebidas", "Bebidas y aguas")
            .to_json();

        assert!(json.starts_with('{'));
        assert!(json.contains("\"tipo\":\"categoria\""));
        assert!(json.contains("\"descripcion\":\"Bebidas\""));
        assert!(json.contains("\"campo\":\"categoria\""));
        assert!(json.contains("\"antes\":\"Bebidas\""));
        assert!(json.contains("\"valor\":\"Bebidas y aguas\""));
    }

    #[test]
    fn opt_str_muestra_guion_cuando_none() {
        assert_eq!(opt_str(&None), "-");
        assert_eq!(opt_str(&Some("abc".to_string())), "abc");
    }
}