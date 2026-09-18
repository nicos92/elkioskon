use chrono::Timelike;

use crate::domain::entities::{es_horario_nocturno, HoraConfig, NocturnoConfig};

pub fn margen_efectivo(
    ganancia: f64,
    ganancia_diurna: f64,
    ganancia_nocturna: f64,
    activo: bool,
    es_nocturno: bool,
) -> f64 {
    if !activo {
        return ganancia;
    }
    if es_nocturno {
        if ganancia_nocturna > 0.0 {
            return ganancia_nocturna;
        }
    } else if ganancia_diurna > 0.0 {
        return ganancia_diurna;
    }
    ganancia
}

pub fn es_nocturno_ahora(config: &NocturnoConfig) -> Option<bool> {
    let inicio = HoraConfig::from_hhmm(&config.hora_inicio)?;
    let fin = HoraConfig::from_hhmm(&config.hora_fin)?;
    if inicio.minutos_desde_medianoche() == fin.minutos_desde_medianoche() {
        return Some(false);
    }
    let now = chrono::Local::now();
    let ahora_minutos = now.hour() * 60 + now.minute();
    Some(es_horario_nocturno(
        ahora_minutos,
        inicio.minutos_desde_medianoche(),
        fin.minutos_desde_medianoche(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactivo_siempre_usa_ganancia_general() {
        assert_eq!(margen_efectivo(20.0, 10.0, 30.0, false, false), 20.0);
        assert_eq!(margen_efectivo(20.0, 10.0, 30.0, false, true), 20.0);
    }

    #[test]
    fn nocturno_activo_usado_cuando_es_nocturno() {
        assert_eq!(margen_efectivo(20.0, 10.0, 30.0, true, true), 30.0);
    }

    #[test]
    fn diurno_activo_usado_fuera_del_rango_nocturno() {
        assert_eq!(margen_efectivo(20.0, 10.0, 30.0, true, false), 10.0);
    }

    #[test]
    fn nocturno_sin_valor_cae_a_ganancia_general() {
        assert_eq!(margen_efectivo(20.0, 10.0, 0.0, true, true), 20.0);
    }

    #[test]
    fn diurno_sin_valor_cae_a_ganancia_general() {
        assert_eq!(margen_efectivo(20.0, 0.0, 30.0, true, false), 20.0);
    }
}