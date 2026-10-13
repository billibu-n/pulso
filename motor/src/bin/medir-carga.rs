// Mide cuanto TARDA abrir, con y sin memoria. Sin esto, «la cache acelera» es una creencia.
//
// SE CONSERVA, como los otros instrumentos de medida del proyecto, porque **este numero hay que poder
// volver a medirlo**: si algun dia se cambia `Real` y la memoria deja de valer, aqui se ve.
//
// MEDIDO ANTES DE ESCRIBIR LA MEMORIA (2026-10-13, 102 ficheros de verdad):
//   leer los 102 ficheros .... 5,50 s   <-- TODO el tiempo esta aqui
//   contar 549.842 muestras .. 0,000 s
//   calcular la deriva ....... 0,00 s
// O sea que el problema nunca fue calcular: era VOLVER A LEER.
//
// Y MEDIDO DESPUES, que es lo que justifica el modulo `memoria`:
//   sin memoria (primera vez) ... 5,40 s
//   con memoria (segunda) ....... 0,001 s   ==> 6.586 veces mas rapido
//   el fichero de memoria ....... 9.473 bytes (nueve kilobytes, en vez de releer 1.003 MB)
//
// Uso:  cargo run -q --release --bin medir-carga -- ../datos/garmin_files/storage
use std::path::{Path, PathBuf};
use std::time::Instant;

fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or_else(|| "../datos/garmin_files/storage".into());
    let carpeta = Path::new(&carpeta);
    let memoria = PathBuf::from("/tmp/pulso-memoria-prueba.json");
    let _ = std::fs::remove_file(&memoria);

    println!("--- 1) SIN memoria (primera vez, como hoy) ---");
    let t0 = Instant::now();
    let real = motor::analisis::de_carpeta(carpeta);
    let sin = t0.elapsed();
    println!("  leer + calcular ...... {:.2} s", sin.as_secs_f64());

    println!("\n--- 2) se guarda la memoria ---");
    let t1 = Instant::now();
    motor::memoria::guardar(&memoria, carpeta, &real);
    println!("  guardar .............. {:.3} s", t1.elapsed().as_secs_f64());
    let tam = std::fs::metadata(&memoria).map(|m| m.len()).unwrap_or(0);
    println!("  el fichero de memoria  {tam} bytes");

    println!("\n--- 3) CON memoria (segunda apertura) ---");
    let t2 = Instant::now();
    let leido = motor::memoria::leer(&memoria, carpeta);
    let con = t2.elapsed();
    println!("  leer la memoria ...... {:.3} s", con.as_secs_f64());

    match leido {
        Some(r) => {
            println!("  y el resultado es el mismo: {} sesiones, {} muestras, deriva {:?}",
                r.sesiones, r.muestras, r.deriva_media);
            assert_eq!(r, real, "la memoria tiene que devolver LO MISMO");
            println!("\n  ==> {:.2} s -> {:.3} s  ·  {:.0} veces mas rapido",
                sin.as_secs_f64(), con.as_secs_f64(), sin.as_secs_f64() / con.as_secs_f64().max(0.0001));
        }
        None => println!("  ¡LA MEMORIA NO SIRVIO! Se volveria a leer todo."),
    }
    let _ = std::fs::remove_file(&memoria);
}
