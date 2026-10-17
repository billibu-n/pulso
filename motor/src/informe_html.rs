//! **El mismo informe, en HTML**: para leerlo en un navegador y mandarlo.
//!
//! POR QUE EXISTE (T006, 2026-10-17): el informe ya se generaba en Markdown, que esta bien para el
//! que lo lee en un editor y **no sirve para el cliente**: no lo abre, o lo abre en un editor que
//! le ensena las almohadillas. El HTML es el formato que SI se abre con doble clic y se imprime.
//!
//! QUE **NO** HACE, y es lo importante: **no piensa ni redacta**. Convierte a HTML el Markdown que
//! ya salio de `informe::generar`, que a su vez solo coloca numeros calculados. Si este modulo
//! inventara una frase, el informe dejaria de ser auditable. Por eso la unica transformacion es de
//! FORMA (titulos, tablas, negritas), y esta cubierta por pruebas.
//!
//! SIN DEPENDENCIAS EXTERNAS: ni CSS de un CDN, ni una linea de JavaScript. El fichero se abre
//! **sin conexion**, que es como se va a abrir en un portatil en una casa.

/// Escapa lo que el navegador interpretaria como marcado.
///
/// NO ES OPCIONAL: los nombres de fichero del cliente llevan `&` y `_`, y un `<` suelto rompe la
/// pagina entera o, peor, inyecta marcado. Se escapa SIEMPRE, antes de poner negritas.
pub fn escapar(t: &str) -> String {
    let mut o = String::with_capacity(t.len());
    for c in t.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&#39;"),
            _ => o.push(c),
        }
    }
    o
}

/// Negritas (`**`) y cursivas (`*`) de UNA linea, ya escapada. Es lo unico de Markdown que usa el
/// informe, y por eso no se mete un conversor entero: una dependencia de mas es una promesa de
/// divergencia.
fn en_linea(t: &str) -> String {
    // SE ESCAPA AQUI, una sola vez, y de ahi sale todo lo demas: asi no hay ninguna rama del
    // conversor que pueda olvidarse de escapar (lo cazo una prueba: un nombre con `<` se colaba).
    let t = escapar(t);
    let mut o = String::with_capacity(t.len());
    let mut resto = t.as_str();
    while let Some(i) = resto.find("**") {
        let (antes, desde) = resto.split_at(i);
        o.push_str(antes);
        let tras = &desde[2..];
        match tras.find("**") {
            Some(j) => {
                o.push_str("<strong>");
                o.push_str(&tras[..j]);
                o.push_str("</strong>");
                resto = &tras[j + 2..];
            }
            None => {
                o.push_str("**");
                resto = tras;
            }
        }
    }
    o.push_str(resto);

    // Cursivas: a estas alturas no queda ninguna pareja de `**`, asi que todo `*` es cursiva.
    let mut o2 = String::with_capacity(o.len());
    let mut resto = o.as_str();
    while let Some(i) = resto.find('*') {
        let (antes, desde) = resto.split_at(i);
        o2.push_str(antes);
        let tras = &desde[1..];
        match tras.find('*') {
            Some(j) => {
                o2.push_str("<em>");
                o2.push_str(&tras[..j]);
                o2.push_str("</em>");
                resto = &tras[j + 1..];
            }
            None => {
                o2.push('*');
                resto = tras;
            }
        }
    }
    o2.push_str(resto);
    o2
}

/// Las celdas de una fila de tabla Markdown: `| a | b |` → `["a", "b"]`.
fn celdas(linea: &str) -> Vec<String> {
    linea
        .trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|c| c.trim().to_string())
        .collect()
}

/// ¿La linea es el separador de una tabla (`|---|---:|`)?
fn es_separador(linea: &str) -> bool {
    let l = linea.trim();
    l.starts_with('|') && l.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

/// Convierte el informe Markdown en un documento HTML completo, con su CSS embebido.
pub fn a_html(markdown: &str, titulo: &str, fecha: &str) -> String {
    let lineas: Vec<&str> = markdown.lines().collect();
    let mut cuerpo = String::new();
    let mut i = 0;

    while i < lineas.len() {
        let l = lineas[i];
        let t = l.trim();

        if t.is_empty() {
            i += 1;
            continue;
        }

        // ---- Titulos ----
        if let Some(resto) = t.strip_prefix("## ") {
            cuerpo.push_str(&format!("<h2>{}</h2>\n", en_linea(resto)));
            i += 1;
            continue;
        }
        if let Some(resto) = t.strip_prefix("# ") {
            cuerpo.push_str(&format!("<h1>{}</h1>\n", en_linea(resto)));
            i += 1;
            continue;
        }

        // ---- Regla horizontal ----
        if t == "---" {
            cuerpo.push_str("<hr>\n");
            i += 1;
            continue;
        }

        // ---- Tabla ----
        if t.starts_with('|') {
            let mut filas: Vec<&str> = Vec::new();
            while i < lineas.len() && lineas[i].trim().starts_with('|') {
                filas.push(lineas[i].trim());
                i += 1;
            }
            cuerpo.push_str("<table>\n");
            let mut cuerpo_filas = 0;
            for (k, fila) in filas.iter().enumerate() {
                if k == 0 {
                    cuerpo.push_str("<thead><tr>");
                    for c in celdas(fila) {
                        cuerpo.push_str(&format!("<th>{}</th>", en_linea(&c)));
                    }
                    cuerpo.push_str("</tr></thead>\n<tbody>\n");
                } else if k == 1 && es_separador(fila) {
                    continue;
                } else {
                    cuerpo.push_str("<tr>");
                    for c in celdas(fila) {
                        cuerpo.push_str(&format!("<td>{}</td>", en_linea(&c)));
                    }
                    cuerpo.push_str("</tr>\n");
                    cuerpo_filas += 1;
                }
            }
            cuerpo.push_str("</tbody>\n</table>\n");
            // Una tabla sin filas es una tabla vacia: se dice, no se deja un marco.
            if cuerpo_filas == 0 {
                cuerpo.push_str("<p class=\"nota\">(tabla sin filas)</p>\n");
            }
            continue;
        }

        // ---- Lista ----
        if t.starts_with("- ") {
            cuerpo.push_str("<ul>\n");
            while i < lineas.len() {
                let lt = lineas[i].trim();
                match lt.strip_prefix("- ") {
                    Some(r) => {
                        cuerpo.push_str(&format!("<li>{}</li>\n", en_linea(r)));
                        i += 1;
                    }
                    None => break,
                }
            }
            cuerpo.push_str("</ul>\n");
            continue;
        }

        // ---- Parrafo (o la nota de una tabla, que empieza por `:`) ----
        let clase = if t.starts_with(": ") { " class=\"nota\"" } else { "" };
        let texto = t.strip_prefix(": ").unwrap_or(t);
        cuerpo.push_str(&format!("<p{clase}>{}</p>\n", en_linea(texto)));
        i += 1;
    }

    let mut h = String::with_capacity(cuerpo.len() + 2048);
    h.push_str("<!DOCTYPE html>\n<html lang=\"es\">\n<head>\n<meta charset=\"utf-8\">\n");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    h.push_str(&format!("<title>{}</title>\n", escapar(titulo)));
    h.push_str("<style>\n");
    h.push_str(
        ":root{--tinta:#14181f;--suave:#4b5563;--linea:#d7dbe2;--fondo:#ffffff;--papel:#f6f7f9;--acento:#1e5eff}\n\
         *{box-sizing:border-box}\n\
         body{margin:0;background:var(--papel);color:var(--tinta);font:16px/1.55 system-ui,-apple-system,\"Segoe UI\",Roboto,Arial,sans-serif}\n\
         main{max-width:860px;margin:32px auto;padding:32px 40px;background:var(--fondo);border:1px solid var(--linea);border-radius:12px}\n\
         h1{margin:0 0 4px;font-size:28px;line-height:1.2}\n\
         h2{margin:32px 0 12px;font-size:20px;line-height:1.25;border-bottom:2px solid var(--linea);padding-bottom:6px}\n\
         p{margin:12px 0}\n\
         .fecha{margin:0 0 24px;color:var(--suave);font-size:13px}\n\
         .nota{color:var(--suave);font-size:13px}\n\
         table{width:100%;border-collapse:collapse;margin:12px 0 16px;font-size:14px}\n\
         th,td{padding:8px 10px;border-bottom:1px solid var(--linea);text-align:left;vertical-align:top}\n\
         th{background:var(--papel);font-weight:700}\n\
         td:last-child,th:last-child{text-align:left}\n\
         ul{margin:12px 0 12px 22px;padding:0}\n\
         li{margin:6px 0}\n\
         hr{margin:28px 0;border:0;border-top:1px solid var(--linea)}\n\
         footer{margin-top:28px;padding-top:12px;border-top:1px solid var(--linea);color:var(--suave);font-size:12px}\n\
         @media print{body{background:#fff}main{margin:0;border:0;border-radius:0;padding:0;max-width:none}}\n",
    );
    h.push_str("</style>\n</head>\n<body>\n<main>\n");
    h.push_str(&format!("<h1>{}</h1>\n", escapar(titulo)));
    if !fecha.is_empty() {
        h.push_str(&format!("<p class=\"fecha\">{}</p>\n", escapar(fecha)));
    }
    h.push_str(&cuerpo);
    h.push_str("<footer>Generado por Pulso. Cada numero sale de un calculo del motor; ");
    h.push_str("este documento no es un diagnostico medico.</footer>\n</main>\n</body>\n</html>\n");
    h
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn escapa_lo_que_el_navegador_interpretaria() {
        assert_eq!(escapar("<script>"), "&lt;script&gt;");
        assert_eq!(escapar("a & b"), "a &amp; b");
        assert_eq!(escapar("\"x\""), "&quot;x&quot;");
    }

    #[test]
    fn las_negritas_y_las_cursivas_se_convierten() {
        assert_eq!(en_linea("La **referencia** es *su* umbral"), "La <strong>referencia</strong> es <em>su</em> umbral");
        // Un asterisco suelto NO se come el texto.
        assert_eq!(en_linea("1 * 2 = 2"), "1 * 2 = 2");
    }

    #[test]
    fn un_titulo_y_un_parrafo() {
        let h = a_html("# Analisis\n\nHola **mundo**.\n", "Informe", "2026-10-17");
        assert!(h.contains("<h1>Analisis</h1>"));
        assert!(h.contains("<p>Hola <strong>mundo</strong>.</p>"));
        assert!(h.contains("<title>Informe</title>"));
    }

    #[test]
    fn la_tabla_sale_con_cabecera_y_filas() {
        let md = "## Zonas\n\n| zona | pulsaciones |\n|---|---:|\n| **Z2** | 120 – 140 |\n";
        let h = a_html(md, "T", "");
        assert!(h.contains("<thead><tr><th>zona</th><th>pulsaciones</th></tr></thead>"));
        assert!(h.contains("<td><strong>Z2</strong></td><td>120 – 140</td>"));
        assert!(!h.contains("|---"));
    }

    #[test]
    fn una_lista_sale_como_lista() {
        let h = a_html("- edad\n- peso\n", "T", "");
        assert!(h.contains("<ul>"));
        assert!(h.contains("<li>edad</li>"));
        assert_eq!(h.matches("<li>").count(), 2);
    }

    #[test]
    fn el_documento_es_completo_y_sin_dependencias() {
        let h = a_html("# x\n", "T", "2026-10-17");
        assert!(h.starts_with("<!DOCTYPE html>"));
        assert!(h.contains("</html>"));
        assert!(!h.contains("<script"));
        assert!(!h.contains("http://"));
        assert!(!h.contains("https://"));
        assert!(h.contains("charset=\"utf-8\""));
    }

    #[test]
    fn un_nombre_de_fichero_con_menor_no_rompe_la_pagina() {
        let h = a_html("Fichero <malo>.fit\n", "T", "");
        assert!(h.contains("Fichero &lt;malo&gt;.fit"));
        assert!(!h.contains("<malo>"));
    }
}
