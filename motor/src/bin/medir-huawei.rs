//! **Mide lo que trae DE VERDAD una exportacion de Huawei, sin decidir nada.**
//!
//! CONTRATO (cinco lineas), como todo instrumento de este proyecto:
//! - RECIBE: la ruta de un ZIP de Huawei (tal cual sale del correo) o de una carpeta descomprimida.
//! - DEVUELVE, por pantalla: cuantas mediciones hay, su periodo, y por campo cuantas la traen y su
//!   minimo y su maximo. Nada mas.
//! - NO HACE: no decide, no avisa, no estima ni rellena un campo ausente; no escribe en disco; no
//!   conoce a \`Persona\` ni a \`Hoy\`; no abre ventana.
//!
//! POR QUE EXISTE: la pregunta abierta de este bloque es si la bascula del cliente rellena de verdad
//! \`moistureRate\`, \`muscleMass\` y \`boneSalt\`. \`pulso --texto\` solo ensena el agua; este
//! instrumento ensena LOS CINCO campos, con cuantos vienen y su rango. Es lo que hay que mirar en SU
//! ZIP el dia que llegue, con un comando y sin teclear nada a mano.
//!
//! USO:
//!   cargo run -q --bin medir-huawei -- <su.zip o su carpeta>

use motor::datos::{Composicion, Dato, Fecha};
use motor::huawei;
use std::collections::BTreeMap;

fn main() {
    let argumentos: Vec<String> = std::env::args().collect();
    if argumentos
        .iter()
        .any(|a| a == "--ayuda" || a == "-h" || a == "--help")
    {
        imprimir_ayuda();
        return;
    }
    let Some(ruta) = argumentos.get(1) else {
        eprintln!("Falta la ruta: el ZIP de Huawei o la carpeta ya descomprimida.");
        std::process::exit(2);
    };
    match huawei::leer(std::path::Path::new(ruta)) {
        Ok(composiciones) => print!("{}", resumen(&composiciones)),
        Err(e) => {
            eprintln!("No se pudo leer «{ruta}»: {e}");
            std::process::exit(1);
        }
    }
}

fn imprimir_ayuda() {
    let p = env!("CARGO_BIN_NAME");
    println!("{p}: mide que trae de verdad una exportacion de Huawei, sin decidir nada.");
    println!();
    println!("USO: {p} <ZIP-o-carpeta-de-Huawei>");
    println!("  Imprime: mediciones, periodo y, por campo, cuantas la traen y su minimo y maximo.");
}

/// **La tabla, ya escrita. Pura y probada**: los mismos datos dan el mismo texto.
fn resumen(composiciones: &[Composicion]) -> String {
    let mut s = String::new();
    s.push_str("--- LO QUE TRAE DE VERDAD LA EXPORTACION DE HUAWEI ---\n");
    s.push_str(&format!("  mediciones ................ {}\n", composiciones.len()));
    if composiciones.is_empty() {
        s.push_str("  (no traia ninguna medicion de composicion)\n");
    }

    let fechas: Vec<Fecha> = composiciones.iter().filter_map(|c| c.fecha.valor()).collect();
    s.push_str(&format!(
        "  con fecha ................. {} de {}\n",
        fechas.len(),
        composiciones.len()
    ));
    if let (Some(primera), Some(ultima)) = (
        fechas.iter().min_by_key(|f| clave(f)),
        fechas.iter().max_by_key(|f| clave(f)),
    ) {
        s.push_str(&format!(
            "  periodo ................... {} .. {}\n",
            primera.a_texto(),
            ultima.a_texto()
        ));
        let mut por_dia: BTreeMap<(u16, u8, u8), usize> = BTreeMap::new();
        for f in &fechas {
            *por_dia.entry(clave(f)).or_insert(0) += 1;
        }
        let max = por_dia.values().copied().max().unwrap_or(0);
        s.push_str(&format!(
            "  dias distintos ............ {}\n  maximo en un dia .......... {}\n",
            por_dia.len(),
            max
        ));
    }

    s.push_str("\n  campo (clave JSON)             con dato     minimo      maximo\n");
    let columnas = [
        ("peso_kg", "bodyWeight", 1, valores(composiciones, |c| c.peso_kg)),
        ("grasa_corporal_%", "bodyFatRate", 1, valores(composiciones, |c| c.grasa_corporal_pct)),
        ("agua_corporal_%", "moistureRate", 1, valores(composiciones, |c| c.agua_corporal_pct)),
        ("masa_muscular_kg", "muscleMass", 1, valores(composiciones, |c| c.masa_muscular_kg)),
        ("masa_osea_kg", "boneSalt", 2, valores(composiciones, |c| c.masa_osea_kg)),
    ];
    for (nombre, json, dec, vs) in &columnas {
        let con = format!("{}/{}", vs.len(), composiciones.len());
        let (mini, maxi) = if vs.is_empty() {
            ("—".to_string(), "—".to_string())
        } else {
            let min = vs.iter().copied().fold(f32::INFINITY, f32::min);
            let max = vs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            (num(min, *dec), num(max, *dec))
        };
        let etiqueta = format!("{nombre} ({json})");
        s.push_str(&format!("  {etiqueta:<30} {con:<11} {mini:<11} {maxi}\n"));
    }

    // Y SE DICE LO QUE NO HAY, en una linea: es exactamente la pregunta de este bloque.
    let faltan: Vec<&str> = columnas
        .iter()
        .filter(|c| c.3.is_empty())
        .map(|c| c.0)
        .collect();
    if !faltan.is_empty() {
        s.push_str(&format!(
            "\n  SIN DATO EN NINGUNA MEDICION: {}\n",
            faltan.join(", ")
        ));
    }
    s
}

/// La clave de orden de una fecha: la terna (ano, mes, dia), que es como se comparan los dias.
fn clave(f: &Fecha) -> (u16, u8, u8) {
    (f.anio, f.mes, f.dia)
}

/// Los valores PRESENTES de un campo. Lo que no esta **no entra**, y por eso el recuento no miente.
fn valores(composiciones: &[Composicion], cual: fn(&Composicion) -> Dato<f32>) -> Vec<f32> {
    composiciones.iter().filter_map(|c| cual(c).valor()).collect()
}

/// Un numero con coma decimal, como se escribe en espanol.
fn num(v: f32, decimales: usize) -> String {
    format!("{v:.decimales$}").replace('.', ",")
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn opcion<T>(v: Option<T>) -> Dato<T> {
        match v {
            Some(x) => Dato::Hay(x),
            None => Dato::Falta,
        }
    }

    fn con(
        fecha: Option<Fecha>,
        peso: Option<f32>,
        agua: Option<f32>,
        musculo: Option<f32>,
        osea: Option<f32>,
    ) -> Composicion {
        Composicion {
            fecha: opcion(fecha),
            peso_kg: opcion(peso),
            grasa_corporal_pct: Dato::Falta,
            agua_corporal_pct: opcion(agua),
            masa_muscular_kg: opcion(musculo),
            masa_osea_kg: opcion(osea),
        }
    }

    #[test]
    fn cuenta_los_campos_que_vienen_y_su_rango_sin_inventar_nada() {
        let cs = vec![
            con(Some(Fecha::nueva(2026, 10, 1)), Some(70.0), Some(55.0), Some(30.0), Some(3.0)),
            con(Some(Fecha::nueva(2026, 10, 10)), Some(72.0), None, None, Some(3.4)),
        ];
        let t = resumen(&cs);
        assert!(t.contains("mediciones ................ 2"), "{t}");
        assert!(t.contains("peso_kg (bodyWeight)"), "{t}");
        assert!(t.contains("2/2"), "el peso viene en las dos: {t}");
        assert!(t.contains("agua_corporal_% (moistureRate)"), "{t}");
        assert!(t.contains("3,0") && t.contains("3,4"), "min y max de masa osea: {t}");
    }

    #[test]
    fn un_campo_ausente_se_dice_que_no_esta_y_no_se_dibuja_un_cero() {
        let cs = vec![con(Some(Fecha::nueva(2026, 10, 1)), Some(70.0), None, None, None)];
        let t = resumen(&cs);
        assert!(t.contains("0/1"), "el agua no viene: {t}");
        assert!(t.contains("SIN DATO EN NINGUNA MEDICION"), "{t}");
        // La fila del agua, mirada ENTERA: ausente se dice con un guion, nunca con un cero. (No vale
        // buscar «0,0» en todo el texto: «70,0» lo contiene y la prueba pasaria por casualidad.)
        let fila = t
            .lines()
            .find(|l| l.contains("agua_corporal_%"))
            .expect("tiene que estar la fila del agua");
        assert!(
            fila.contains('—') && !fila.contains(','),
            "un campo ausente NO puede salir con un numero: {fila}"
        );
    }

    #[test]
    fn el_periodo_es_el_min_y_el_max_y_cuenta_los_dias() {
        let cs = vec![
            con(Some(Fecha::nueva(2026, 1, 5)), Some(70.0), None, None, None),
            con(Some(Fecha::nueva(2026, 10, 10)), Some(70.0), None, None, None),
            con(Some(Fecha::nueva(2026, 10, 10)), Some(71.0), None, None, None),
        ];
        let t = resumen(&cs);
        assert!(t.contains("2026-01-05 .. 2026-10-10"), "{t}");
        assert!(t.contains("dias distintos ............ 2"), "{t}");
        assert!(t.contains("maximo en un dia .......... 2"), "{t}");
    }
}
