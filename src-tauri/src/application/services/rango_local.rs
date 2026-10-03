use chrono::{DateTime, Local, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};

use crate::infrastructure::error::AppError;

/// Tope de intentos para resolver una hora local que no existe.
///
/// Un salto de DST adelante mueve el reloj una hora. El margen de 24 existe
/// para los casos históricos extremos del tz database (Samoa saltó el 30 de
/// diciembre de 2011 entero). El objetivo es que la función **termine**
/// siempre, no que tenga éxito siempre.
const MAX_HORAS_DE_BUSQUEDA: i64 = 24;

/// Rango UTC `[inicio, fin)` que cubre un día local completo, en RFC 3339.
///
/// Se resuelve la medianoche **local** y no un día UTC porque las ventas y los
/// cierres se registran con hora local: "hoy" no es un día UTC. El `fin` es
/// exclusivo, así que una venta exacta a la medianoche del día siguiente ya
/// cuenta para el día siguiente.
///
/// ## Por qué esto no puede entrar en panic
///
/// `from_local_datetime` devuelve tres casos, y el código anterior hacía
/// `.earliest().or_else(..).unwrap()`, que entra en panic en el tercero:
///
/// - `Single`: la hora existe una vez.
/// - `Ambiguous`: el reloj retrocede y la hora existe dos veces. Se toma la
///   primera, porque el día arranca con la primera aparición.
/// - `None`: el reloj adelanta y **la hora no existe**.
///
/// El tercer caso es real y recurrente en el tz database. Midiendo 2000-2044
/// contra las zonas IANA (`chrono-tz`, test `medianoche_inexistente_en_chile` y
/// el escaneo del que salió):
///
/// | Zona | Medianoches inexistentes | Próxima |
/// | ---- | ------------------------ | ------- |
/// | `America/Santiago` | 44 | todos los años, hasta 2044-09-04 |
/// | `America/Havana` | 43 | todos los años, hasta 2044-03-13 |
/// | `Africa/Cairo` | 36 | todos los años, hasta 2044-04-29 |
/// | `Asia/Beirut` | 45 | todos los años, hasta 2044-03-27 |
/// | `America/Asuncion` | 25 | último: 2024-10-06 |
/// | `America/Argentina/Buenos_Aires` | 2 | ninguno; la última fue 2008-10-19 |
///
/// Argentina es el único caso de la lista que ya no expone, así que el riesgo
/// real es una máquina configurada en otra zona (Chile y Cuba son vecinas, y un
/// usuario que viaja lleva su reloj). Para `None` se busca la primera hora que sí
/// existe: según el reloj, el día arranca cuando el reloj salta. La alternativa
/// —devolver un error— convertiría esas noches en un dashboard con error y un día
/// que no se puede cerrar, que es justo cuando el cierre importa.
pub fn rango_utc_del_dia_local(day: NaiveDate) -> Result<(String, String), AppError> {
    rango_utc_del_dia_local_en(&Local, day)
}

/// El rango para una zona arbitraria. `rango_utc_del_dia_local` lo llama con
/// `Local`; existe para que los tests puedan ejercitar las ramas del `match` con
/// datos IANA reales en vez de constructores sintéticos.
fn rango_utc_del_dia_local_en<Tz: TimeZone>(
    tz: &Tz,
    day: NaiveDate,
) -> Result<(String, String), AppError> {
    let manana = day
        .checked_add_days(chrono::Days::new(1))
        .ok_or_else(|| AppError::Internal(format!("Fecha fuera de rango: {}", day)))?;

    Ok((medianoche_a_utc(tz, day)?, medianoche_a_utc(tz, manana)?))
}

fn medianoche_a_utc<Tz: TimeZone>(tz: &Tz, day: NaiveDate) -> Result<String, AppError> {
    // `and_hms_opt` y no `and_hms` para no tener un panic en el código: la API
    // que devuelve `Option` obliga a decidir, y para 00:00:00 siempre decide
    // bien. Si mañana se cambia la hora, el compilador señala el sitio.
    let medianoche = day
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| AppError::Internal(format!("Medianoche local inválida: {}", day)))?;

    Ok(resolver_instante_local(tz, medianoche)?
        .with_timezone(&Utc)
        .to_rfc3339())
}

/// Primer instante real que corresponde a esa hora de pared.
fn resolver_instante_local<Tz: TimeZone>(
    tz: &Tz,
    medianoche: NaiveDateTime,
) -> Result<DateTime<Tz>, AppError> {
    match tz.from_local_datetime(&medianoche) {
        LocalResult::Single(instante) => Ok(instante),
        LocalResult::Ambiguous(primero, _) => Ok(primero),
        LocalResult::None => {
            let mut probe = medianoche;
            for _ in 0..MAX_HORAS_DE_BUSQUEDA {
                probe += chrono::Duration::hours(1);
                match tz.from_local_datetime(&probe) {
                    LocalResult::Single(instante) | LocalResult::Ambiguous(instante, _) => {
                        return Ok(instante)
                    }
                    LocalResult::None => continue,
                }
            }
            Err(AppError::Internal(format!(
                "No se resolvió la hora local {} tras {} intentos de una hora",
                medianoche, MAX_HORAS_DE_BUSQUEDA
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::Tz;

    /// Chile salta de 23:59 a 01:00 el primer domingo de septiembre, así que la
    /// medianoche local no existe. Esta es la rama que antes era un panic, y se
    /// prueba contra el tz database real, no contra un doble de test.
    #[test]
    fn medianoche_inexistente_en_chile_arranca_en_la_primera_hora_existente() {
        let tz = Tz::America__Santiago;
        let dia = NaiveDate::from_ymd_opt(2026, 9, 6).expect("fecha válida");

        // Premisa: la medianoche de ese día no existe en esa zona.
        let medianoche = dia.and_hms_opt(0, 0, 0).expect("medianoche");
        assert!(
            matches!(tz.from_local_datetime(&medianoche), LocalResult::None),
            "la premisa del test no se sostiene: 2026-09-06 00:00 sí existe en Santiago"
        );

        let (inicio, fin) = rango_utc_del_dia_local_en(&tz, dia).expect("no debe fallar");

        let instante: DateTime<Utc> = DateTime::parse_from_rfc3339(&inicio)
            .expect("RFC 3339 válido")
            .with_timezone(&Utc);
        let local = instante.with_timezone(&tz);

        assert_eq!(
            local.date_naive(),
            dia,
            "el rango sigue siendo del día pedido"
        );
        assert_eq!(
            local.time(),
            chrono::NaiveTime::from_hms_opt(1, 0, 0).expect("hora válida"),
            "con un hueco a medianoche el día arranca a la 01:00"
        );
        assert!(
            DateTime::parse_from_rfc3339(&fin).expect("RFC 3339 válido") > instante,
            "el rango no puede estar vacío"
        );
    }

    /// La prueba de regresión de verdad: el código anterior, con el mismo día y
    /// la misma zona, entraba en panic. Si alguien vuelve a poner el `.unwrap()`
    /// sobre `.earliest().or_else(..)`, este test lo detecta.
    #[test]
    fn el_codigo_anterior_entaba_en_panic() {
        let tz = Tz::America__Santiago;
        let dia = NaiveDate::from_ymd_opt(2026, 9, 6).expect("fecha válida");
        let dt = dia.and_hms_opt(0, 0, 0).expect("medianoche");

        let resultado = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tz.from_local_datetime(&dt)
                .earliest()
                .or_else(|| tz.from_local_datetime(&dt).latest())
                .unwrap()
        }));

        assert!(
            resultado.is_err(),
            "se esperaba el panic del código anterior; si esto pasa, la premisa \
             del test (medianoche inexistente) cambió en el tz database"
        );
    }

    /// Cuba retrocede el reloj a la medianoche, así que la hora existe dos veces.
    /// Se tiene que tomar la primera: el día arranca con la primera aparición.
    #[test]
    fn medianoche_ambigua_toma_la_primera_aparicion() {
        let tz = Tz::America__Havana;
        let dia = NaiveDate::from_ymd_opt(2026, 11, 1).expect("fecha válida");

        let medianoche = dia.and_hms_opt(0, 0, 0).expect("medianoche");
        let LocalResult::Ambiguous(primero, segundo) = tz.from_local_datetime(&medianoche) else {
            panic!("la premisa del test no se sostiene: 2026-11-01 00:00 no es ambiguo");
        };
        assert!(primero < segundo, "la primera aparición es la más temprana");

        let (inicio, _) = rango_utc_del_dia_local_en(&tz, dia).expect("no debe fallar");
        let inicio: DateTime<Utc> = DateTime::parse_from_rfc3339(&inicio)
            .expect("RFC 3339 válido")
            .with_timezone(&Utc);

        assert_eq!(inicio, primero.with_timezone(&Utc), "se toma la primera");
    }

    /// Un día sin cambio es la midnight local exacta. Es el caso que importa
    /// todos los días y el que no se puede romper.
    #[test]
    fn un_dia_sin_cambio_empieza_a_la_medianoche_local() {
        for (tz, dia) in [
            (Tz::America__Argentina__Buenos_Aires, (2026, 6, 15)),
            (Tz::America__Santiago, (2026, 6, 15)),
            (Tz::UTC, (2026, 6, 15)),
        ] {
            let dia = NaiveDate::from_ymd_opt(dia.0, dia.1, dia.2).expect("fecha válida");
            let (inicio, _) = rango_utc_del_dia_local_en(&tz, dia).expect("no debe fallar");

            let instante: DateTime<Utc> = DateTime::parse_from_rfc3339(&inicio)
                .expect("RFC 3339 válido")
                .with_timezone(&Utc);

            assert_eq!(
                instante.with_timezone(&tz).date_naive(),
                dia,
                "{:?}: el inicio debe caer en el día pedido",
                tz
            );
        }
    }

    /// El invariante que vale para cualquier zona y cualquier día, incluidos los
    /// que tienen hueco o ambigüedad: el rango es semiabierto, no vacío, y
    /// pegado al rango del día siguiente.
    #[test]
    fn el_frontera_entre_dias_es_compartida_en_cualquier_zona() {
        for tz in [
            Tz::America__Santiago,
            Tz::America__Havana,
            Tz::America__Argentina__Buenos_Aires,
            Tz::UTC,
        ] {
            for dia in [
                NaiveDate::from_ymd_opt(2026, 6, 15).expect("fecha válida"),
                // Con hueco a medianoche.
                NaiveDate::from_ymd_opt(2026, 9, 6).expect("fecha válida"),
                // Con ambigüedad a medianoche.
                NaiveDate::from_ymd_opt(2026, 11, 1).expect("fecha válida"),
            ] {
                let siguiente = dia
                    .checked_add_days(chrono::Days::new(1))
                    .expect("suma válida");

                let (_, fin) = rango_utc_del_dia_local_en(&tz, dia).expect("no debe fallar");
                let (inicio, _) =
                    rango_utc_del_dia_local_en(&tz, siguiente).expect("no debe fallar");

                assert_eq!(
                    fin, inicio,
                    "{:?} {}: el fin de un día debe ser el inicio del siguiente",
                    tz, dia
                );
            }
        }
    }

    /// Cubre el camino real de `get_home_stats`, que usa la fecha de hoy y la
    /// zona del sistema.
    #[test]
    fn funciona_para_hoy_en_la_zona_del_sistema() {
        let hoy = Local::now().date_naive();
        let (inicio, fin) = rango_utc_del_dia_local(hoy).expect("el rango de hoy es resoluble");

        let desde: DateTime<Utc> = DateTime::parse_from_rfc3339(&inicio)
            .expect("RFC 3339 válido")
            .with_timezone(&Utc);
        let hasta: DateTime<Utc> = DateTime::parse_from_rfc3339(&fin)
            .expect("RFC 3339 válido")
            .with_timezone(&Utc);

        assert!(desde < hasta, "el rango de hoy no puede estar vacío");
    }
}
