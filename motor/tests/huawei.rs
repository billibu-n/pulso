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

// ─────────────────────────────────────────────────────────────────────────────────────────────
// El ZIP: abrirlo es trabajo de la APLICACION, no del cliente
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Una caja temporal propia. Cada prueba usa un nombre distinto: corren en paralelo.
fn caja(nombre: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("pulso-huawei-{}-{}", std::process::id(), nombre));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("crear la caja");
    d
}

fn escribir_zip(ruta: &std::path::Path, entradas: &[(&str, &str)]) {
    use std::io::Write;
    let f = std::fs::File::create(ruta).expect("crear el zip");
    let mut w = zip::ZipWriter::new(f);
    let opciones = zip::write::SimpleFileOptions::default();
    for (nombre, contenido) in entradas {
        w.start_file(*nombre, opciones).expect("entrada del zip");
        w.write_all(contenido.as_bytes()).expect("escribir la entrada");
    }
    w.finish().expect("cerrar el zip");
}

#[test]
fn abre_el_zip_de_huawei_sin_descomprimirlo() {
    let caja = caja("zip");
    let ruta = caja.join("HUAWEI_HEALTH_20260101.zip");
    escribir_zip(
        &ruta,
        &[(
            "HUAWEI_HEALTH_20260101/Health detail data & description/health detail data123.json",
            EJEMPLO_COMPLETO,
        )],
    );
    let v = huawei::leer(&ruta).expect("el ZIP tiene que leerse tal cual, sin descomprimir");
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].agua_corporal_pct, Dato::Hay(57.1), "la clave del ZIP se encontró a cualquier profundidad");
    let _ = std::fs::remove_dir_all(&caja);
}

#[test]
fn una_carpeta_ya_descomprimida_tambien_se_lee() {
    let caja = caja("carpeta");
    let sub = caja.join("Health detail data & description");
    std::fs::create_dir_all(&sub).expect("subcarpeta");
    std::fs::write(sub.join("health detail data999.json"), EJEMPLO_COMPLETO).expect("json");
    let v = huawei::leer(&caja).expect("la carpeta tiene que leerse igual");
    assert_eq!(v.len(), 1);
    let _ = std::fs::remove_dir_all(&caja);
}

#[test]
fn un_zip_sin_datos_de_composicion_lo_dice() {
    let caja = caja("vacio");
    let ruta = caja.join("sin-datos.zip");
    escribir_zip(&ruta, &[("leeme.txt", "aqui no hay salud")]);
    assert!(matches!(
        huawei::leer(&ruta),
        Err(huawei::ErrorHuawei::SinFicheros(_))
    ));
    let _ = std::fs::remove_dir_all(&caja);
}

#[test]
fn un_fichero_que_no_es_un_zip_lo_dice_y_no_revienta() {
    let caja = caja("nozip");
    let ruta = caja.join("no-es-un.zip");
    std::fs::write(&ruta, b"esto no es un zip").expect("escribir");
    assert!(matches!(
        huawei::leer(&ruta),
        Err(huawei::ErrorHuawei::NoEsUnZip(_))
    ));
    let _ = std::fs::remove_dir_all(&caja);
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// EL PUENTE COMPLETO: el agua que entra al recorrido CON SU FECHA
// ─────────────────────────────────────────────────────────────────────────────────────────────

#[test]
fn el_agua_leida_de_Huawei_entra_al_recorrido_CON_SU_FECHA() {
    // Es la prueba que sostiene el contrato docs/CONTRATO-agua-en-el-recorrido.md: un JSON minimo
    // inventado -> huawei::leer_texto -> Hoy::de_composiciones -> el agua con su dia. Sin la aplicacion
    // y sin un solo dato del cliente.
    let composiciones = huawei::leer_texto(EJEMPLO_COMPLETO).expect("el JSON se lee");
    let hoy = motor::avisos::Hoy::de_composiciones(&composiciones);
    assert_eq!(hoy.agua_corporal_pct, Dato::Hay(57.1));
    assert_eq!(
        hoy.fecha_agua,
        Dato::Hay(Fecha::nueva(2023, 11, 15)),
        "el dia sale del startTime y su zona, y viaja con el valor"
    );
}
