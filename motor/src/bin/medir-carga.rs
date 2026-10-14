// Mide cuanto TARDA abrir, y cuanto se RELEE. Sin esto, «la cache acelera» es una creencia.
//
// SE CONSERVA, como los otros instrumentos de medida del proyecto, porque **estos numeros hay que poder
// volver a medirlos**: si algun dia se cambia lo que se guarda y la memoria deja de valer, aqui se ve.
//
// MEDIDO ANTES DE ESCRIBIR LA MEMORIA (2026-10-13, 102 ficheros de verdad):
//   leer los 102 ficheros .... 5,50 s   <-- TODO el tiempo esta aqui
//   contar 549.842 muestras .. 0,000 s
//   calcular la deriva ....... 0,00 s
// O sea que el problema nunca fue calcular: era VOLVER A LEER.
//
// Y MEDIDO DESPUES (2026-10-14), con la memoria INCREMENTAL de fichas:
//   primera vez (lee todo) ......... el tiempo de leer los FIT
//   segunda (no cambia nada) ....... milisegundos, y RELEE 0 FICHEROS
//   con una ficha menos (llega una sesion nueva) ... lee SOLO ESA, y el total no cambia
//
// Uso:  cargo run -q --release --bin medir-carga -- ../datos/garmin_files/storage
use std::path::{Path, PathBuf};
use std::time::Instant;

fn main() {
    let carpeta = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../datos/garmin_files/storage".into());
    let carpeta = Path::new(&carpeta);
    let memoria = PathBuf::from("/tmp/pulso-memoria-incremental.json");
    let _ = std::fs::remove_file(&memoria);

    println!("--- 1) SIN memoria (primera vez: lo lee todo) ---");
    let t0 = Instant::now();
    let (real, fichas, c1) = motor::analisis::actualizar(carpeta, &[]);
    let sin = t0.elapsed();
    println!(
        "  leer + calcular ...... {:.2} s   ({} fichas, {} leidas)",
        sin.as_secs_f64(),
        fichas.len(),
        c1.leidas
    );

    println!("\n--- 2) se guarda la memoria (una ficha por fichero) ---");
    let t1 = Instant::now();
    motor::memoria::guardar(&memoria, &fichas);
    println!("  guardar .............. {:.3} s", t1.elapsed().as_secs_f64());
    let tam = std::fs::metadata(&memoria).map(|m| m.len()).unwrap_or(0);
    println!("  el fichero de memoria  {tam} bytes");

    println!("\n--- 3) CON memoria, sin cambios (NO debe releer ninguno) ---");
    let previas = motor::memoria::leer_fichas(&memoria);
    let t2 = Instant::now();
    let (leido, _fichas2, c2) = motor::analisis::actualizar(carpeta, &previas);
    let con = t2.elapsed();
    println!(
        "  leer la memoria ...... {:.3} s   ({} recordadas, {} leidas)",
        con.as_secs_f64(),
        c2.recordadas,
        c2.leidas
    );
    assert_eq!(leido, real, "la memoria tiene que devolver LO MISMO");
    assert_eq!(c2.leidas, 0, "sin cambios NO se puede releer ni un fichero");

    println!("\n--- 4) llega UNA sesion nueva (simulada: falta una ficha) ---");
    let mut sin_una = previas.clone();
    let _quitada = sin_una.pop();
    let t3 = Instant::now();
    let (real3, _f, c3) = motor::analisis::actualizar(carpeta, &sin_una);
    println!(
        "  tiempo ............... {:.3} s   ({} recordadas, {} leidas)",
        t3.elapsed().as_secs_f64(),
        c3.recordadas,
        c3.leidas
    );
    assert_eq!(c3.leidas, 1, "solo se puede releer la que falta");
    assert_eq!(real3, real, "y el total tiene que ser el mismo");

    println!(
        "\n  ==> {:.2} s -> {:.3} s ({} veces mas rapido), y una sesion nueva cuesta {:.3} s",
        sin.as_secs_f64(),
        con.as_secs_f64(),
        sin.as_secs_f64() / con.as_secs_f64().max(0.0001),
        t3.elapsed().as_secs_f64()
    );
    if let (Some(desde), Some(hasta)) = (real.desde, real.hasta) {
        println!("  periodo cubierto ...... epoch {desde} .. {hasta}");
    } else {
        println!("  periodo cubierto ...... SIN FECHAS en los ficheros");
    }
    let _ = std::fs::remove_file(&memoria);
}
