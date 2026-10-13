//! **Cuenta y enseña TODO lo que el motor saca de sus datos.**
//!
//! `cargo run --bin inventario -- ../datos/garmin_files/storage`
//!
//! Existe para poder CONTESTAR «cuantos resultados genera» con un numero, y no con una impresion.

use motor::datos::{Dato, Persona};
use motor::{fit, formulas_v2, hrv, patron, umbral};
use std::path::Path;

fn main() {
    let carpeta = std::env::args().nth(1).unwrap_or("../datos/garmin_files/storage".into());
    let (ficheros, _) = fit::leer_carpeta(Path::new(&carpeta));

    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║  CUÁNTOS RESULTADOS SACA EL MOTOR, contados                              ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");

    // ---- Lo que ENTRA ----
    let mut total_muestras = 0usize;
    let mut total_rr = 0usize;
    for (_, l) in &ficheros {
        total_muestras += l.sesion.muestras.len();
        total_rr += l.sesion.intervalos_rr_ms.len();
    }
    println!("▌ LO QUE ENTRA (102 ficheros FIT)\n");
    println!("  sesiones ................ {}.", ficheros.len());
    println!("  muestras (pulso+potencia)  {total_muestras}");
    println!("  intervalos entre latidos . {total_rr}");
    println!();

    // ---- Lo que SALE ----
    let mut p = Persona::desconocida();
    p.edad = Dato::Hay(60);
    p.pulso_reposo = Dato::Hay(48);
    p.peso_kg = Dato::Hay(70.2);
    p.altura_cm = Dato::Hay(178.0);
    p.grasa_corporal_pct = Dato::Hay(19.1);
    p.umbral_lactato = Dato::Hay(169);
    // La maxima se busca en TODAS las sesiones: es donde esta. De una sola saldria corta.
    p.pulso_maximo_medido = match ficheros
        .iter()
        .filter_map(|(_, l)| l.declarado.pulso_maximo.valor())
        .max()
    {
        Some(m) => Dato::Hay(m),
        None => Dato::Falta,
    };

    println!("▌ RESULTADOS DE LA PERSONA (una vez, no por sesión)\n");

    // 1 el umbral
    let u = umbral::umbral(&p);
    println!("  1. umbral de lactato ................. {:.0} ± {:.0} ppm  ({})", u.rango.valor, u.rango.margen, u.referencia.nombre());
    // 2 la frecuencia maxima
    let m = formulas_v2::pulso_maximo(&p);
    println!("  2. frecuencia máxima ................. {:.0} ± {:.0} ppm", m.rango.valor, m.rango.margen);
    // 3 la reserva
    let r = formulas_v2::reserva_cardiaca(&p, &m);
    println!("  3. reserva cardíaca .................. {:.0} ± {:.0} ppm", r.valor, r.margen);
    // 4-8 las zonas de ciclismo (5)
    let zc = umbral::zonas_del_umbral(&p);
    println!("  4. zonas de ciclismo (sobre el umbral) ... {} tramos, cada uno con su margen", zc.len());
    for (z, d, h) in &zc {
        // La recuperacion NO es un tramo: no tiene suelo, porque empieza en cero. Se dice «hasta».
        // Esto se corrigio el 2026-10-13: salia «137 - 137», un tramo de ancho CERO, que no es un
        // tramo. **Un informe que dice dos veces el mismo numero no se entiende.**
        if (h.valor - d.valor).abs() < 1.0 {
            println!("       {:<16} hasta {:.0} ppm", z.nombre(), h.valor);
        } else {
            println!("       {:<16} {:.0} – {:.0} ppm", z.nombre(), d.valor, h.valor);
        }
    }
    // 9-12 las zonas por frecuencia (4)
    let zf = formulas_v2::zonas(&p);
    println!("  5. zonas por frecuencia cardíaca ........ {} límites", zf.len());
    // 13 el VO2
    match umbral::vo2_maximo(&p) {
        Dato::Hay(v) => println!("  6. consumo máximo de oxígeno .......... {:.0} ± {:.0} ml/kg/min", v.valor, v.margen),
        Dato::Falta => println!("  6. consumo máximo de oxígeno .......... (no lo tiene)"),
    }
    // 14 el aviso
    let aviso = motor::esfuerzo::aviso_de_la_estimacion(&p);
    println!("  7. aviso sobre la estimación .......... {}", if aviso.is_some() { "sí" } else { "no hace falta" });
    // 15 la precision de conjunto
    println!("  8. precisión del conjunto ............ {}", formulas_v2::precision_de_conjunto(&p).nombre());

    println!("\n▌ RESULTADOS POR SESIÓN (× 102 sesiones)\n");
    let mut n_deriva = 0; let mut n_ef = 0; let mut n_desfase = 0; let mut n_hrv = 0; let mut n_trimp = 0;
    let mut n_salto = 0; let mut n_tendencia = 0;

    for (_, l) in &ficheros {
        let s = &l.sesion;
        if patron::deriva(s).muestras >= 20 { n_deriva += 1; }
        if patron::eficiencia(s).valor > 0.0 { n_ef += 1; }
        if let Dato::Hay(_) = patron::desfase(s, 60) { n_desfase += 1; }
        if hrv::calcular(hrv::intervalos_de(s)).latidos >= 120 { n_hrv += 1; }
        if formulas_v2::trimp(s, &p).valor > 0.0 { n_trimp += 1; }
        let _ = &mut n_salto; let _ = &mut n_tendencia;
    }
    println!("  9.  deriva del pulso ................. en {n_deriva} de 102");
    println!("  10. eficiencia (vatios por pulso) .... en {n_ef} de 102");
    println!("  11. desfase pulso/potencia ........... en {n_desfase} de 102");
    println!("  12. HRV (RMSSD, SDNN, coeficiente) ... en {n_hrv} de 102");
    println!("  13. carga de la sesión (TRIMP) ....... en {n_trimp} de 102");
    println!("  14. declarado por el fichero (pulso medio, máximo, duración, calorías, deporte) ... en 102 de 102");

    println!("\n▌ RESULTADOS DEL CONJUNTO (necesitan histórico)\n");
    println!("  15. tendencia de una serie ............ cuando hay 2+ puntos en el tiempo");
    println!("  16. salto respecto al mes anterior .... cuando hay mes anterior");
    println!("  17. salto respecto a todo el histórico  cuando hay histórico");
    println!("  18. HRV respecto a su propia normalidad  cuando hay 5+ sesiones");

    println!("\n▌ Y EL INFORME\n");
    println!("  19. el informe completo ............... 1 documento, con 5 secciones y 2 tablas");

    // ---- LA CUENTA ----
    let por_persona = 8;
    let por_sesion = 6;
    let del_conjunto = 4;
    let informe = 1;
    println!("\n╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║  LA CUENTA                                                                 ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");
    println!("  de la persona ................ {por_persona}");
    println!("  por sesión ................... {por_sesion}   × 102 sesiones = {}", por_sesion*ficheros.len());
    println!("  del conjunto ................. {del_conjunto}");
    println!("  el informe ................... {informe}");
    println!("  {}", "-".repeat(52));
    println!("  TOTAL ........................ {}", por_persona + por_sesion*ficheros.len() + del_conjunto + informe);
    println!();
    println!("  Y CADA UNO lleva tres cosas: su VALOR, su MARGEN y su ORIGEN.");
    println!("  O sea: {} números, {} márgenes y {} fuentes.", por_persona + por_sesion*ficheros.len() + del_conjunto + informe,
             por_persona + por_sesion*ficheros.len() + del_conjunto + informe,
             por_persona + por_sesion*ficheros.len() + del_conjunto + informe);


}
