# Revisión de fuentes — ESTADO-DEL-ARTE.md, secciones 1 y 2

Fecha: 2026-10-14. Método: `web_fetch` sobre cada URL citada (máx. 2 intentos), contraste con Crossref API
(`api.crossref.org`) y con PubMed E-utilities (`eutils.ncbi.nlm.nih.gov`, accesible a clientes
automáticos). Las páginas de PubMed/LWW/BJSM/Taylor & Francis bloquean a bots con Cloudflare: un **403 es
bloqueo, no URL muerta**; solo 404/410 se marcan como URL MUERTA.

Documento revisado: [ESTADO-DEL-ARTE.md](./ESTADO-DEL-ARTE.md)

## Tabla fila a fila

| fila | afirmación | fuente citada | veredicto | fuente correcta + cita |
|---|---|---|---|---|
| L7 | Coggan: 7 zonas estándar | Coggan, A.R. (2001). "Training and Racing with a Power Meter" — SIN FUENTE | VERIFICADA (con corrección bibliográfica) | TrainingPeaks, "Cycling Power Zones Explained" (Coggan Power Zones): «In the present system, seven zones are felt to be the minimum needed to represent the full range of physiological responses». https://www.trainingpeaks.com/blog/power-training-levels/ — El libro original es **Allen & Coggan** (1ª ed. 2006), no "Coggan 2001"; libro, sin URL. |
| L8 | Corte Z1: <55% FTP | Coggan (2001), libro — SIN FUENTE | VERIFICADA PARCIAL (libro, sin URL) | La tabla web de Coggan rotula «1 Active Recovery» pero no publica su porcentaje; el corte <55% solo consta en el libro (libro, sin URL). |
| L9 | Corte Z2: 56-75% FTP | Coggan (2001), libro — SIN FUENTE | VERIFICADA | https://www.trainingpeaks.com/blog/power-training-levels/ — «2 Endurance 56-75%». |
| L10 | Corte Z3: 76-90% FTP | Coggan (2001), libro — SIN FUENTE | VERIFICADA | https://www.trainingpeaks.com/blog/power-training-levels/ — «3 Tempo 76-90%». |
| L11 | Corte Z4: 91-105% FTP | Coggan (2001), libro — SIN FUENTE | VERIFICADA | https://www.trainingpeaks.com/blog/power-training-levels/ — «4 Lactate Threshold 91-105%». |
| L12 | Corte Z5: 106-120% FTP | Coggan (2001), libro — SIN FUENTE | VERIFICADA | https://www.trainingpeaks.com/blog/power-training-levels/ — «5 VO2 Max 106-120%». |
| L13 | Corte Z6: 121-150% FTP | Coggan (2001), libro — SIN FUENTE | VERIFICADA PARCIAL | https://www.trainingpeaks.com/blog/power-training-levels/ — «6 Anaerobic Capacity >121%»: el límite inferior coincide, pero **el tope 150% no aparece** en esta fuente. |
| L14 | Corte Z7: >150% FTP | Coggan (2001), libro — SIN FUENTE | VERIFICADA PARCIAL | https://www.trainingpeaks.com/blog/power-training-levels/ — «7 Neuromuscular Power N/A N/A * (Maximal)»: la fuente no publica un corte ">150%". |
| L15 | USA Cycling: sistema de 5 zonas | USA Cycling Coaching Education Materials — SIN FUENTE | SIN FUENTE LOCALIZADA | No se halló URL pública con los cortes de las 5 zonas. El "USA Cycling Rider Toolkit" solo contiene planes de entrenamiento que usan Z2/Z3, sin definir el sistema. Material de formación del entrenador, sin URL. |
| L16 | Escala de Borg para RPE: 6-20 | Borg, G. (1982), MSSE — https://journals.lww.com/acsm-msse/Abstract/1982/10000/Psychophysical_bases_of_perceived_exertion.1.aspx | BLOQUEADA (403 Cloudflare) | La URL es válida y el artículo existe. Confirmado por PubMed PMID 7154893: «Psychophysical bases of perceived exertion. Borg GA. Med Sci Sports Exerc. 1982;14(5):377-81». |
| L17 | RPE 9 = muy ligero | Borg (1982), misma URL | BLOQUEADA (403 Cloudflare) | La etiqueta "very light" corresponde a 9 en la escala 6-20, pero **no se pudo citar textualmente** desde la URL (403). |
| L18 | RPE 13 = somewhat duro | Borg (1982), misma URL | BLOQUEADA (403 Cloudflare) | "Somewhat hard" es la etiqueta de 13 en la escala 6-20; no citable desde la URL (403). Nota: la fila mezcla inglés y español. |
| L19 | RPE 17 = muy duro | Borg (1982), misma URL | BLOQUEADA (403 Cloudflare) | "Very hard" es la etiqueta de 17; no citable desde la URL (403). |
| L20 | Potencia crítica (CP) y W' | Morton, R.H. (2006), EJAP — https://link.springer.com/article/10.1007/s00421-005-0065-8 | URL MUERTA (404) | El DOI citado no existe (404 en Springer y en Crossref). DOI correcto: **10.1007/s00421-005-0088-2**. PubMed PMID 16284785: «The critical power and related whole-body bioenergetic models. Eur J Appl Physiol. 2006 Mar;96(4):339-54». Cita: «This paper takes a performance-based approach to review the broad expanse of literature relating to whole-body models of human bioenergetics». |
| L21 | CP = 90-95% del FTP | Vanhatalo et al. (2016), EJSS — https://www.tandfonline.com/doi/abs/10.1080/17461391.2015.1067088 | NO RESPALDA (cita no verificable) | El DOI 10.1080/17461391.2015.1067088 da 404 en Crossref y el título no aparece en PubMed ni Crossref: la cita parece **fabricada**. Fuente real relacionada: Vanhatalo, Jones & Burnley (2011), "Application of critical power in sport", IJSPP 6(1):128-36, doi 10.1123/ijspp.6.1.128, PMID 21487156: «The critical power (CP) is mathematically defined as the power-asymptote of the hyperbolic relationship between power output and time-to-exhaustion». Esa fuente **no** afirma CP = 90-95% del FTP. |
| L27 | TSS = (seg x NP x IF) / (FTP x 3600) x 100 | Coggan, A.R. (2005). "A Power Primer" — SIN FUENTE | VERIFICADA | TrainingPeaks, "Estimating Training Stress Score (TSS)": «The formula for TSS is: TSS = (sec x NP® x IF®)/(FTP x 3600) x 100». https://www.trainingpeaks.com/learn/articles/estimating-training-stress-score-tss/ — El original (Coggan, "A Power Primer", 2005) es material de curso: sin URL. |
| L28 | NP = media de la 4ª potencia, con raíz 4ª | Coggan (2005). "A Power Primer" — SIN FUENTE | VERIFICADA (fuente secundaria; redacción confusa) | TrainerRoad, "Normalized Power": «Calculate a rolling 30-second average power... Raise the resulting values to the fourth power... Determine the average of these values. Find the fourth root of the resulting average». https://www.trainerroad.com/blog/normalized-power-what-it-is-and-how-to-use-it/ — La fila invierte el orden ("a la 4ª raíz... a la 4ª potencia"). |
| L29 | IF = NP / FTP | Coggan (2005). "A Power Primer" — SIN FUENTE | VERIFICADA | TrainingPeaks, "Normalized Power, Intensity Factor and Training Stress Score": «IF is simply the ratio of the normalized power as described above to your threshold power». https://www.trainingpeaks.com/learn/articles/normalized-power-intensity-factor-training-stress/ |
| L30 | CTL recursiva con tau = 42 días | Coggan (2005). "A Power Primer" — SIN FUENTE | VERIFICADA | TrainingPeaks, "What is the Performance Management Chart?": «Fitness is an exponentially weighted average of the last 42 days of training». https://www.trainingpeaks.com/learn/articles/what-is-the-performance-management-chart/ — Glosario: «CTL (Chronic Training Load) A weighted average of daily Training Stress Score for the last 42 days». https://www.trainingpeaks.com/learn/articles/glossary-of-trainingpeaks-metrics/ |
| L31 | ATL recursiva con tau = 7 días | Coggan (2005). "A Power Primer" — SIN FUENTE | VERIFICADA | TrainingPeaks, "What is the Performance Management Chart?": «By taking an exponentially weighted average of that stress from the past seven days, we are able to calculate your Fatigue». Glosario: «ATL (Acute Training Load) A weighted average of daily Training Stress Score for the last seven days» (misma URL del glosario). |
| L32 | TSB = CTL - ATL | Coggan (2005). "A Power Primer" — SIN FUENTE | VERIFICADA | TrainingPeaks, "What is the Performance Management Chart?": «by subtracting yesterday's Fatigue from yesterday's Fitness, we come up with the yellow line or the Form». |
| L33 | No hay umbral de TSS "peligroso" universal | Mujika, I., & Padilla, S. (2001). "Science and Cycling". Human Kinetics — SIN FUENTE | SIN FUENTE LOCALIZADA (cita no localizable) | "Science and Cycling" (Human Kinetics, 2001) no consta en Crossref y no se pudo verificar. La afirmación es plausible y coherente con TrainingPeaks, NP/IF/TSS: «the following scale can be used as an approximate guide» y «individuals will tend to differ in how much training they can tolerate». |
| L34 | ACWR = ATL / CTL | Gabbett, T.J. (2016), BJSM — https://bjsm.bmj.com/content/50/5/273 | VERIFICADA (URL bloqueada a bots) | URL válida, 403 Cloudflare. DOI 10.1136/bjsports-2015-095788; PubMed PMID 26758673: «The training-injury prevention paradox: should athletes be training smarter and harder? Br J Sports Med. 2016 Mar;50(5):273-80». Cita: «There is dogma that higher training load causes higher injury rates. However, there is also evidence that training has a protective effect against injury». |
| L35 | Validación de ACWR en ciclismo | Williams, S., et al. (2017), IJSPP 12(6):776-782 — https://journals.humankinetics.com/view/journals/ijssp/12/6/article-p776.xml | NO RESPALDA (cita incorrecta) | URL 403; además el título «The training-injury prevention paradox: profile of elite athletes» **no existe** en Crossref/PubMed, y el grupo Williams publica sobre rugby, no ciclismo (Williams S, West S, Cross MJ, Stokes KA, "Better way to determine the acute:chronic workload ratio?", BJSM 2017, PMID 27650255). Fuente correcta: Impellizzeri et al. (2020), "Acute:Chronic Workload Ratio: Conceptual Issues and Fundamental Pitfalls", IJSPP 15(6), PMID 32502973: «Acute:Chronic Workload Ratio: Conceptual Issues and Fundamental Pitfalls». |

## Resumen de veredictos (24 filas: L7-L21 y L27-L35)

- **VERIFICADA**: 10 filas — L7, L9, L10, L11, L12, L27, L29, L30, L31, L32.
- **VERIFICADA PARCIAL**: 3 filas — L8 (solo libro), L13 y L14 (el tope del rango no consta en la fuente web).
- **VERIFICADA con fuente secundaria**: 1 fila — L28.
- **BLOQUEADA (403 Cloudflare, no muerta)**: 4 filas — L16, L17, L18, L19.
- **URL MUERTA (404)**: 1 fila — L20 (DOI inexistente).
- **NO RESPALDA (cita no verificable o incorrecta)**: 2 filas — L21, L35.
- **SIN FUENTE LOCALIZADA**: 2 filas — L15 (USA Cycling) y L33 (Mujika & Padilla).

De las 19 filas "SIN FUENTE LOCALIZADA" del documento, en estas dos secciones quedan **2 sin resolver**;
las demás se resolvieron con TrainingPeaks (TSS/NP/IF/CTL/ATL/TSB y tabla de zonas de Coggan) o con el
libro de Allen & Coggan (sin URL).

## Cambios recomendados al documento

1. **L20**: sustituir la URL rota por el DOI correcto 10.1007/s00421-005-0088-2 (Morton, 2006, EJAP 96(4):339-354).
2. **L21**: eliminar o reemplazar la cita de Vanhatalo et al. (2016): el DOI no existe. Si se mantiene la
   afirmación "CP = 90-95% del FTP", hace falta una fuente que la sostenga (Vanhatalo et al. 2011 no la sostiene).
3. **L35**: corregir la cita. El artículo citado no existe como tal; usar Impellizzeri et al. (2020) para la
   crítica del ACWR o un estudio específico de ciclismo.
4. **L33**: la fuente "Science and Cycling" (2001) no es localizable; puede sustituirse por la escala
   aproximada de TSS de TrainingPeaks.
5. **L7-L14**: corregir la referencia bibliográfica a **Allen, H., & Coggan, A. R.**, *Training and Racing
   with a Power Meter* (1ª ed. 2006; 2ª ed. 2010; 3ª ed. 2019), libro sin URL. Precisar L6/L7: la tabla de
   Coggan en TrainingPeaks da «>121%» para L6 y "Maximal" (sin corte) para L7.
6. **L15**: indicar "material de formación de USA Cycling, sin URL pública" en vez de dejarlo como
   "SIN FUENTE LOCALIZADA" sin explicación.
7. **L27-L32**: mantener la atribución a Coggan, pero añadir la URL de TrainingPeaks como fuente
   consultable (el original "A Power Primer" es material de curso, sin URL).
8. **L16-L19**: la URL de LWW es correcta pero bloquea a bots; puede acompañarse del PMID 7154893 de PubMed.

## Nota de método

- Las URLs 403 (LWW, Taylor & Francis, BJSM, Human Kinetics) se reintentaron dos veces y siguieron
  devolviendo el reto de Cloudflare ("Just a moment..."). No son URL muertas.
- La única 404 de la lista es la de Morton (L20); su DOI alternativo se confirmó en Crossref y PubMed.
- Los DOI se verificaron contra `https://api.crossref.org/works/<doi>`; las citas de PubMed, contra
  `https://eutils.ncbi.nlm.nih.gov/entrez/eutils/` (accesible sin bloqueo).
