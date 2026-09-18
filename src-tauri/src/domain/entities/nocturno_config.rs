use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoraConfig {
    pub hora: u32,
    pub minuto: u32,
}

impl HoraConfig {
    pub fn from_hhmm(hhmm: &str) -> Option<Self> {
        let (hora, minuto) = hhmm.split_once(':')?;
        let hora: u32 = hora.parse().ok()?;
        let minuto: u32 = minuto.parse().ok()?;
        if hora > 23 || minuto > 59 {
            return None;
        }
        Some(Self { hora, minuto })
    }

    pub fn minutos_desde_medianoche(&self) -> u32 {
        self.hora * 60 + self.minuto
    }

    pub fn to_hhmm_string(&self) -> String {
        format!("{:02}:{:02}", self.hora, self.minuto)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NocturnoConfig {
    pub activo: bool,
    pub hora_inicio: String,
    pub hora_fin: String,
}

impl Default for NocturnoConfig {
    fn default() -> Self {
        Self {
            activo: false,
            hora_inicio: "22:00".to_string(),
            hora_fin: "06:00".to_string(),
        }
    }
}

pub fn es_horario_nocturno(
    ahora_minutos: u32,
    inicio_minutos: u32,
    fin_minutos: u32,
) -> bool {
    if inicio_minutos == fin_minutos {
        return false;
    }
    if inicio_minutos < fin_minutos {
        ahora_minutos >= inicio_minutos && ahora_minutos < fin_minutos
    } else {
        ahora_minutos >= inicio_minutos || ahora_minutos < fin_minutos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hora_config_parses_valid_hhmm() {
        let h = HoraConfig::from_hhmm("22:30").unwrap();
        assert_eq!(h.hora, 22);
        assert_eq!(h.minuto, 30);
        assert_eq!(h.minutos_desde_medianoche(), 22 * 60 + 30);
        assert_eq!(h.to_hhmm_string(), "22:30");
    }

    #[test]
    fn hora_config_rejects_invalid_hhmm() {
        assert!(HoraConfig::from_hhmm("25:00").is_none());
        assert!(HoraConfig::from_hhmm("12:60").is_none());
        assert!(HoraConfig::from_hhmm("1234").is_none());
        assert!(HoraConfig::from_hhmm("hola").is_none());
    }

    #[test]
    fn nocturno_dentro_del_mismo_dia() {
        assert!(es_horario_nocturno(23 * 60 + 10, 22 * 60, 6 * 60));
        assert!(es_horario_nocturno(60 + 30, 22 * 60, 6 * 60));
        assert!(!es_horario_nocturno(10 * 60, 22 * 60, 6 * 60));
    }

    #[test]
    fn nocturno_sin_cruce_de_medianoche() {
        assert!(es_horario_nocturno(3 * 60, 2 * 60, 4 * 60));
        assert!(!es_horario_nocturno(60, 2 * 60, 4 * 60));
        assert!(!es_horario_nocturno(5 * 60, 2 * 60, 4 * 60));
    }

    #[test]
    fn rango_inicio_igual_fin_nunca_aplica() {
        assert!(!es_horario_nocturno(0, 3 * 60, 3 * 60));
        assert!(!es_horario_nocturno(2 * 60, 3 * 60, 3 * 60));
    }
}