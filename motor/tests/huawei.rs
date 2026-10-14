//! Las pruebas del adaptador de Huawei, **solas**: no necesitan la aplicación, ni la ventana, ni sus
//! ficheros. Es lo que exige la arquitectura: una pieza que necesita las demás para probarse no está
//! terminada.
//!
//! TODOS los JSON de estas pruebas son **inventados**. El repositorio es público y aquí no entra ni un
//! dato del cliente.

// Los nombres de las pruebas llevan MAYUSCULAS a proposito: lo que se prueba va en el nombre.
#![allow(non_snake_case)]

use motor::datos::{Composicion, Dato, Fecha};
use motor::huawei;

/// Un export mínimo, con la forma real: array de registros, samplePoints, y el value como TEXTO que
/// contiene otro JSON (doble codificación).
const EJEMPLO_COMPLETO: &str = r#"[
  {
    "deviceCode": 4242,
    "timeZone": "+0300",
    "samplePoints": [
      {
        "key": "WEIGHT_BODYFAT_BROAD",
        "startTime": 1700000000000,
        "value": "{\"bodyWeight\":72.5,\"bodyFatRate\":19.4,\"moistureRate\":57.1,\"muscleMass\":31.8,\"boneSalt\":3.12}"
      }
    ]
  }
]"#;

#[test]
fn un_export_completo_da_la_composicion_en_kg_y_porcentaje() {
    let v = huawei::leer_texto(EJEMPLO_COMPLETO).expect("tiene que leerse");
    assert_eq!(v.len(), 1);
    let c: Composicion = v[0];
    assert_eq!(c.peso_kg, Dato::Hay(72.5));
    assert_eq!(c.grasa_corporal_pct, Dato::Hay(19.4));
    assert_eq!(c.agua_corporal_pct, Dato::Hay(57.1), "agua corporal en % (moistureRate)");
    assert_eq!(c.masa_muscular_kg, Dato::Hay(31.8));
    assert_eq!(c.masa_osea_kg, Dato::Hay(3.12));
    // 1700000000000 ms UTC es 2023-11-14T22:13:20Z; con +0300 es ya el dia 15.
    assert_eq!(c.fecha, Dato::Hay(Fecha::nueva(2023, 11, 15)));
}

#[test]
fn sin_agua_el_resto_sigue_y_el_agua_queda_en_FALTA() {
    let texto = r#"[
      {"deviceCode": 1, "timeZone": "+0000", "samplePoints": [
        {"key": "WEIGHT_BODYFAT_BROAD", "startTime": 1700000000000,
         "value": "{\"bodyWeight\":70.0,\"bodyFatRate\":20.0}"}
      ]}
    ]"#;
    let c = huawei::leer_texto(texto).expect("se lee")[0];
    assert_eq!(c.peso_kg, Dato::Hay(70.0), "el peso SI se lee");
    assert_eq!(c.agua_corporal_pct, Dato::Falta, "el agua que no viene NO es un cero");
    assert_eq!(c.masa_osea_kg, Dato::Falta);
}

#[test]
fn un_cero_de_centinela_no_es_un_dato() {
    let texto = r#"[
      {"deviceCode": 1, "timeZone": "+0000", "samplePoints": [
        {"key": "WEIGHT_BODYFAT_BROAD", "startTime": 1700000000000,
         "value": "{\"bodyWeight\":0.0,\"moistureRate\":0.0}"}
      ]}
    ]"#;
    let c = huawei::leer_texto(texto).expect("se lee")[0];
    assert_eq!(c.peso_kg, Dato::Falta, "0 kg es el centinela de un sensor roto");
    assert_eq!(c.agua_corporal_pct, Dato::Falta, "0 % de agua tambien");
}

#[test]
fn el_value_como_objeto_tambien_vale() {
    let texto = r#"[
      {"deviceCode": 1, "timeZone": "+0000", "samplePoints": [
        {"key": "WEIGHT_BODYFAT_BROAD", "startTime": 1700000000000,
         "value": {"bodyWeight": 68.0, "moistureRate": 54.0}}
      ]}
    ]"#;
    let c = huawei::leer_texto(texto).expect("se lee")[0];
    assert_eq!(c.peso_kg, Dato::Hay(68.0));
    assert_eq!(c.agua_corporal_pct, Dato::Hay(54.0));
}

#[test]
fn la_zona_del_registro_manda_en_la_fecha() {
    // El MISMO instante, dos zonas: la fecha local cambia.
    assert_eq!(
        huawei::fecha_desde_ms(1700000000000, "+0000"),
        Dato::Hay(Fecha::nueva(2023, 11, 14))
    );
    assert_eq!(
        huawei::fecha_desde_ms(1700000000000, "+0300"),
        Dato::Hay(Fecha::nueva(2023, 11, 15))
    );
    // Y el cambio de año, que es el caso que una formula de calendario escrita a mano suele fallar.
    assert_eq!(
        huawei::fecha_desde_ms(1704067200000, "+0000"),
        Dato::Hay(Fecha::nueva(2024, 1, 1))
    );
    assert_eq!(
        huawei::fecha_desde_ms(1704067199000, "+0000"),
        Dato::Hay(Fecha::nueva(2023, 12, 31))
    );
}

#[test]
fn el_mismo_punto_dos_veces_no_se_cuenta_dos_veces() {
    // El export repite el dato con type 8 y type 10006; se deduplica por (dispositivo, instante).
    let texto = r#"[
      {"deviceCode": 7, "timeZone": "+0000", "samplePoints": [
        {"key": "WEIGHT_BODYFAT_BROAD", "startTime": 1700000000000, "value": "{\"bodyWeight\":70.0}"},
        {"key": "WEIGHT_BODYFAT_BROAD", "startTime": 1700000000000, "value": "{\"bodyWeight\":70.0}"}
      ]}
    ]"#;
    assert_eq!(huawei::leer_texto(texto).expect("se lee").len(), 1);
}

#[test]
fn sin_puntos_de_composicion_la_lista_esta_vacia_pero_no_falla() {
    let texto = r#"[
      {"deviceCode": 1, "timeZone": "+0000", "samplePoints": [
        {"key": "BLOODPRESSURE", "startTime": 1700000000000, "value": "{\"sphygmus\":60}"}
      ]}
    ]"#;
    assert!(huawei::leer_texto(texto).expect("se lee").is_empty());
}

#[test]
fn un_texto_que_no_es_json_da_error_tipado_y_no_revienta() {
    assert!(huawei::leer_texto("esto no es json").is_err());
}
