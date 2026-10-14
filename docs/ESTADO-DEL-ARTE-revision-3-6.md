# Revisión de fuentes — Secciones 3 a 6 de ESTADO-DEL-ARTE.md

Documento revisado: `/home/billibu/projects/pulso/docs/ESTADO-DEL-ARTE.md` (90 líneas).
Alcance: **sección 3 "Recuperación y sus señales medibles"** (líneas 41-48), **sección 4 "Nutrición e hidratación"** (54-64), **sección 5 "Mecánica de pedaleo y lesión"** (70-79) y **sección 6 "Arquitectura de software"** (85-90). Total: **35 filas**, **25 URL únicas**.

Fecha de la revisión: 2026-10-14.

## Método

1. Cada URL del documento se abrió con `web_fetch` (dos intentos cuando falló por redirección).
2. Un **403 de Cloudflare/editorial no es URL muerta**: se marca BLOQUEADA y no se deduce nada del cuerpo.
3. Para no depender del 403, se contrastó la **existencia real del DOI** con la API Handle de doi.org (`https://doi.org/api/handles/<doi>`, respuesta 1 = existe, 100 = no existe) y los metadatos con **Crossref** (`api.crossref.org`) y **PubMed** (E-utilities).
4. Las citas textuales proceden del **resumen indexado en PubMed** (`efetch`) o de la **página que sí cargó** (Cockburn, blog de Martin). No se inventó ninguna URL ni ninguna cita: donde no hubo acceso al texto se escribe "(sin acceso al texto: 403)".

## Resultado global

| veredicto | filas |
|---|---|
| VERIFICADA (la URL carga y respalda) | 2 |
| BLOQUEADA (403 de Cloudflare; no se pudo leer el texto) | 15 |
| URL MUERTA (HTTP 404/410 o **DOI citado no registrado**) | 10 |
| NO RESPALDA (la URL existe o existiría, pero apunta a **otro** artículo u otro tema) | 5 |
| SIN FUENTE LOCALIZADA (libro físico, sin URL) | 3 |

De las 25 URL únicas: **1 carga** (Cockburn), **19 devuelven 403**, **2 dan 404**, **3 redirigen a un login/dominio ajeno**. Hallazgo importante: **7 de los 403 esconden un DOI que no existe** (no son solo bloqueos, la URL es incorrecta de origen).

---

## Sección 3 — Recuperación y sus señales medibles

| fila | afirmación | fuente citada | veredicto | fuente correcta + cita |
|---|---|---|---|---|
| 41 | Deriva cardíaca: aumento progresivo de FC a potencia constante por deshidratación y flujo | Coyle et al. (1992), ESSR 20:185-216 — journals.lww.com/acsm-essr/.../1992/01000/ | BLOQUEADA (403); además la cita está mal fechada | La obra real es **Coyle EF & González-Alonso J (2001)**, "Cardiovascular Drift During Prolonged Exercise: New Perspectives", ESSR **29(2):88-92**, DOI 10.1249/00003677-200104000-00009 (PMID 11337829). Cita: *"cardiovascular drift, characterized by a progressive decline in stroke volume after 10-20 min of exercise, is primarily due to increased heart rate rather than a progressive increase in cutaneous blood flow as body temperature rises"*. No existe artículo ESSR de 1992 con ese título (PubMed/Crossref). |
| 42 | Cálculo de deriva = ((FC_final - FC_inicial) / FC_inicial) × 100 | Coyle et al. (1992), misma URL | BLOQUEADA (403) | La fórmula no consta en el resumen de la obra correcta (Coyle & González-Alonso 2001, DOI 10.1249/00003677-200104000-00009). No verificado: requiere texto completo. |
| 43 | Umbral significativo de deriva: >5% | Coyle et al. (1992), misma URL | BLOQUEADA (403) | El umbral >5% no aparece en el resumen de Coyle & González-Alonso 2001. No verificado. |
| 44 | HRV (RMSSD): raíz de la media de los cuadrados de las diferencias R-R | Malik et al. (1996), Eur Heart J 17(3):354-381 — academic.oup.com/eurheartj/article/17/3/354/213034 | BLOQUEADA (403); la cita es correcta | DOI verificado: 10.1093/oxfordjournals.eurheartj.a014868 (existe). Texto no accesible (403 de Cloudflare). |
| 45 | Por qué la HRV en ejercicio no es comparable a tablas de población (predomina el simpático) | Akselrod et al. (1981), Science 213(4504):220-222 — science.org/doi/**10.1126/science.7198445** | **URL MUERTA**: el DOI citado no existe | El DOI 10.1126/science.7198445 devuelve responseCode 100 (no registrado). El correcto es **10.1126/science.6166045** (PMID 6166045): *"sympathetic and parasympathetic nervous activity make frequency-specific contributions to the heart rate power spectrum"*. El resumen no afirma nada específico sobre el ejercicio. |
| 46 | Aumento sostenido de pulso en reposo >5 bpm sugiere sobreentrenamiento | Meeusen et al. (2013), MSSE 45(1):186-205 — journals.lww.com/acsm-msse/.../2013/01000/ | BLOQUEADA (403); la cita es correcta | DOI verificado: 10.1249/MSS.0b013e318279a10a (PMID 23247672). Cita del resumen: *"Successful training not only must involve overload but also must avoid the combination of excessive overload plus inadequate recovery"*. El valor ">5 bpm" no consta en el resumen. |
| 47 | Sueño recomendado para atletas: 7-9 h con proporción de profundo y REM | Hirshkowitz et al. (2015), Sleep Health 1(4):233-243 — sleephealthjournal.org | **NO RESPALDA**: volumen/páginas erróneos y población general | La obra es **Sleep Health 2015;1(1):40-43**, DOI 10.1016/j.sleh.2014.12.010 (PMID 29073412). Es una recomendación para **población general**, no específica de atletas; no menciona atletas. |
| 48 | Sueño <6 h aumenta riesgo de lesiones y baja rendimiento | Milewski et al. (2014), JPO 34(2):129-135 — journals.lww.com/jpo/.../2014/03000/ | BLOQUEADA (403); cita casi correcta | Correcto: **JPO 2014;34(2):129-133**, DOI 10.1097/BPO.0000000000000151 (PMID 25028798), título *"Chronic lack of sleep is associated with increased sports injuries in adolescent athletes"*. La página final citada (135) no coincide con 133. |

## Sección 4 — Nutrición e hidratación en deporte de resistencia

| fila | afirmación | fuente citada | veredicto | fuente correcta + cita |
|---|---|---|---|---|
| 54 | Líquido en ejercicio prolongado: 400-800 ml/h individualizado | ACSM (2007), MSSE 39(2):377-390 — journals.lww.com/acsm-msse/.../2007/02000/ | BLOQUEADA (403); la cita es correcta | DOI verificado: 10.1249/mss.0b013e31802ca597 (PMID 17277604). Cita del resumen: *"The goal of drinking during exercise is to prevent excessive (>2% body weight loss from water deficit) dehydration and excessive changes in electrolyte balance"*. Las cifras 400-800 ml/h no constan en el resumen; no verificadas. |
| 55 | Sodio en ejercicio prolongado: 300-600 mg/h | ACSM (2007), misma URL | BLOQUEADA (403) | Mismo artículo (cita correcta, DOI 10.1249/mss.0b013e31802ca597). El rango 300-600 mg/h no consta en el resumen; no verificado. |
| 56 | Agua corporal ≈60% del peso en hombres adultos | Manz & Woldt (2003), EJCN 57(10):1207-1219 — nature.com/articles/1601666 | **NO RESPALDA**: la URL apunta a otro artículo | nature.com/articles/**1601666** corresponde al DOI 10.1038/sj.ejcn.1601666 = **Oku T et al. (2003)**, *"Comparison of digestibility and breath hydrogen gas excretion of fructo-oligosaccharide..."*, EJCN 57:1150-1156. No se localiza la obra "Manz & Woldt (2003)" en PubMed ni Crossref. |
| 57 | Carbohidratos: 30-60 g/h (1-2,5 h) y hasta 90 g/h (>2,5 h) | Jeukendrup & Killer (2010), EJSS 10(3):169-176 — tandfonline.com/doi/abs/**10.1080/17461391.2010.489274** | **URL MUERTA**: el DOI citado no existe | El DOI 10.1080/17461391.2010.489274 devuelve responseCode 100 (no registrado en Handle ni en Crossref). No se localiza la obra con ese título; fuente correcta no identificada. |
| 58 | Carga de carbohidratos: 8-12 g/kg/día, 36-48 h antes | Burke et al. (2011), J Sports Sci 29(S1):S17-S27 — tandfonline.com/doi/abs/**10.1080/02640414.2011.588425** | **URL MUERTA**: el DOI citado no existe | El DOI correcto es **10.1080/02640414.2011.585473**, y sí corresponde a *"Carbohydrates for training and competition"*, J Sports Sci 2011;29(S1):S17-S27, Burke LM; Hawley JA; Wong SH; Jeukendrup AE. Las páginas citadas son correctas; sólo el DOI/URL está mal. |
| 59 | Proteína para deportistas: 1,2-2,0 g/kg/día | Phillips & **Louard** (2011), J Sports Sci 29(S1):S29-S38 — 10.1080/02640414.2011.**619241** | **URL MUERTA** y **autor erróneo** | El DOI 10.1080/02640414.2011.619241 no existe. El correcto es **10.1080/02640414.2011.619204** (PMID 22150425), autores **Phillips SM & Van Loon LJC**, no "Louard". Páginas S29-S38 correctas. El rango 1,2-2,0 g/kg no consta en el resumen. |
| 60 | Sin evidencia sólida de caldo de huesos sobre otras proteínas | Shaw & Lee (2020), Nutrients 12(6):1692 — mdpi.com/2072-6642/12/6/1692 | **NO RESPALDA**: la URL es otro artículo | El DOI 10.3390/nu12061692 (que sí existe) corresponde a **Coyle DH et al. (2020)**, *"The Effects of a Supermarket-Based Intervention on the Nutritional Quality of Private-Label Foods"*, Nutrients 12(6):1692. No es un estudio de colágeno ni de caldo de huesos. |
| 61 | Electrolitos caseros equivalentes a soluciones comerciales con Na y K adecuados | Hew-Butler et al. (2015), Clin J Sport Med 25(4):303-320 — journals.lww.com/cjsm/.../2015/07000/ | BLOQUEADA (403); la cita apunta bien | Obra verificada (PMID 26227507; se publicó también en Br J Sports Med 2015;49(22):1432-46). El resumen no afirma la equivalencia funcional de soluciones caseras; no verificado. |
| 62 | CoQ10: evidencia inconsistente; beneficio mínimo | López-Lluch et al. (2019), Antioxid Redox Signal 31(14):1056-1085 — liebertpub.com/doi/**10.1089/ars.2018.7593** | **URL MUERTA**: el DOI citado no existe | El DOI 10.1089/ars.2018.7593 devuelve responseCode 100 (no registrado). Obras reales del autor: Front Physiol 2018;9:44 (DOI 10.3389/fphys.2018.00044) y Free Radic Biol Med 2021;164:285-302. No se localiza el título citado. |
| 63 | Omega-3: puede reducir inflamación y dolor muscular tardío | Tartibian et al. (2009), Clin J Sport Med 19(2):115-122 — journals.lww.com/cjsm/.../2009/03000/ | BLOQUEADA (403); cita con página final errónea | DOI verificado: 10.1097/JSM.0b013e31819b51b3 (PMID 19451765). Páginas reales **115-119** (no 122). Cita: *"examine the effect of ingestion of omega-3 on perceived pain and external symptoms of delayed onset muscle soreness after eccentric exercise in knee extensors"*. |
| 64 | Calcio/vitamina D: salud ósea, evidencia limitada para recuperación muscular | Larson-Meyer & Willis (2010), Curr Sports Med Rep 9(4):220-226 — journals.lww.com/csmr/.../2010/07000/ | BLOQUEADA (403) | Texto no accesible (403). La cita (revista, año, volumen) es plausible, pero la afirmación no se pudo verificar. |

## Sección 5 — Mecánica de pedaleo y lesión

| fila | afirmación | fuente citada | veredicto | fuente correcta + cita |
|---|---|---|---|---|
| 70 | Fase de pedaleo (power phase): % del ciclo con fuerza positiva | Cukierski & Faria (2004), J Sports Sci 22(12):1091-1097 — 10.1080/02640414.2004.057752 | **URL MUERTA**: el DOI citado no existe | DOI no registrado (Handle 100). El artículo no aparece en Crossref ni en PubMed (el autor "Cukierski" indexado no es de ciclismo). Fuente correcta no localizada. |
| 71 | La fase se mide en % del ciclo completo (0-360 grados) | Cukierski & Faria (2004), misma URL | **URL MUERTA**: el DOI citado no existe | Ídem fila 70. |
| 72 | Valor normal de fase de pedaleo: 60-80% en ciclistas entrenados | Ericson et al. (1986), EJAP 55(2):179-185 — link.springer.com/article/**10.1007/BF00428971** | **NO RESPALDA**: la URL apunta a otro artículo | El DOI 10.1007/BF00428971 **sí existe**, pero es **Patton JF et al. (1982)**, *"Evaluation of a maximal predictive cycle ergometer test of aerobic power"*, EJAP 49:131-140. Obra de Ericson más cercana: *"Muscular activity during ergometer cycling"*, J Rehabil Med 1985;17:53-61 (DOI 10.2340/165019771985175361). |
| 73 | Asimetría normal entre piernas: 0-5% | Paterson et al. (1996), MSSE 28(11):1343-1350 — journals.lww.com/acsm-msse/.../1996/11000/ | BLOQUEADA (403) + cita no localizada | El artículo no aparece en Crossref ni PubMed con ese título y autor. Cita dudosa; no se pudo leer por 403. |
| 74 | >10% de asimetría sostenida indica desequilibrio o riesgo de lesión | Barton et al. (2013), BJSM 47(3):156-163 — bjsm.bmj.com/content/47/3/156 | **NO RESPALDA**: la fuente es de otro tema | El título citado trata de **ortesis plantares** ("custom made foot orthoses"), no de asimetría entre piernas ni de umbral de lesión. La afirmación no está respaldada por esa fuente. |
| 75 | PCO: distancia horizontal entre el punto medio de los tacos y el centro de presión | Hansen et al. (2003), MSSE 35(5):805-811 — journals.lww.com/acsm-msse/.../2003/05000/ | BLOQUEADA (403) + cita no localizada | No se localiza el título "Power-cycling relationship..." en Crossref. Obra de Hansen más cercana: EJAP 2004;91:435-442, DOI 10.1007/s00421-003-0997-x. |
| 76 | Unidad de PCO: milímetros, positivo al exterior | Hansen et al. (2003), misma URL | BLOQUEADA (403) + cita no localizada | Ídem fila 75. |
| 77 | PCO típico en ruta: 0-10 mm al exterior | Hansen et al. (2003), misma URL | BLOQUEADA (403) + cita no localizada | Ídem fila 75. |
| 78 | Cadencia óptima: 80-100 rpm | Pocock et al. (2011), EJAP 111(5):933-942 — link.springer.com/article/**10.1007/s00421-010-1696-2** | **URL MUERTA**: HTTP 404 y el DOI no existe | `web_fetch` devolvió **404 "Page Unavailable"** y el DOI 10.1007/s00421-010-1696-2 devuelve responseCode 100 (no registrado). Fuente correcta no localizada. |
| 79 | Naturaleza discutida de la cadencia óptima: no hay consenso | Rosa et al. (2017), EJSS 17(4):433-440 — 10.1080/17461391.2016.1258921 | **URL MUERTA**: el DOI citado no existe | DOI no registrado (Handle 100). El título no aparece en Crossref ni PubMed. Fuente correcta no localizada. |

## Sección 6 — Arquitectura de software

| fila | afirmación | fuente citada | veredicto | fuente correcta + cita |
|---|---|---|---|---|
| 85 | Arquitectura hexagonal (puertos y adaptadores) formulada por Cockburn en 2005 | Cockburn (2005) — alistair.cockburn.us/hexagonal-architecture/ | **VERIFICADA** | Misma URL (HTTP 200). Cita: *"The Hexagonal (Ports & Adapters) Architecture — HaT Technical Report 2005.02 — Date: 2005-09-04 (v 0.9 to be updated after reader comments)"*. |
| 86 | Los puertos comunican la aplicación con el exterior sin conocer la implementación | Cockburn (2005), misma URL | **VERIFICADA** | Misma URL (HTTP 200). Cita: *"As events arrive from the outside world at a port, a technology-specific adapter converts it into a usable procedure call or message and passes it to the application. The application is blissfully ignorant of the nature of the input device"*. También: *"the application communicates over ports to external agencies"*. |
| 87 | Inversión de dependencias (Martin) | Martin (2000), "Design Principles and Design Patterns" — objectmentor.com/resources/articles/ddp.pdf | **URL MUERTA** | El dominio objectmentor.com ya no sirve el PDF: redirige a **all-hotels.com** (dominio aparcado). Sin captura en Wayback (consulta a la API de disponibilidad: sin snapshots). No se verificó ninguna copia accesible. |
| 88 | Límites (boundaries): dónde termina la responsabilidad de un componente | Martin (2012), Clean Architecture, Prentice Hall | **SIN FUENTE LOCALIZADA** (libro físico, sin URL) | Alternativa accesible del propio autor: `https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html` (HTTP 200). Cita: *"At the lower right of the diagram is an example of how we cross the circle boundaries"*. La definición concreta de la fila no aparece literalmente. |
| 89 | DDD separa el modelo de dominio complejo en capas estratégicas | Evans (2003), Domain-Driven Design, Addison-Wesley | **SIN FUENTE LOCALIZADA** (libro físico, sin URL) | No se localiza edición en línea de la obra. Sin cita textual disponible; no verificado. |
| 90 | Las dependencias apuntan hacia dentro, a mayor abstracción | Martin (2012), Clean Architecture, Prentice Hall | **SIN FUENTE LOCALIZADA** (libro), pero **afirmación verificada en fuente alternativa** | blog.cleancoder.com (2012), HTTP 200. Cita: *"The Dependency Rule. This rule says that source code dependencies can only point inwards"*. La misma página menciona el *"Dependency Inversion Principle"* como mecanismo. |

---

## Aviso sobre los 403 (importante)

Un **403 no es URL muerta**, pero tampoco prueba que la URL sea correcta. En esta revisión, **7 de las URLs que devolvieron 403 encubren un DOI que no existe**:

- fila 45 — 10.1126/science.7198445
- fila 57 — 10.1080/17461391.2010.489274
- fila 58 — 10.1080/02640414.2011.588425
- fila 59 — 10.1080/02640414.2011.619241
- fila 62 — 10.1089/ars.2018.7593
- fila 70 — 10.1080/02640414.2004.057752
- fila 79 — 10.1080/17461391.2016.1258921

Aunque se abrieran en un navegador humano, esas direcciones no resolverían al artículo citado. Comprobación: API Handle de doi.org (responseCode 100 = no registrado).

## Filas que requieren corrección (aunque el texto estuviera bloqueado)

1. **41, 42, 43** — la obra no es de 1992 ni ESSR 20:185-216; es Coyle & González-Alonso, ESSR 2001;29(2):88-92.
2. **45** — DOI inexistente; usar 10.1126/science.6166045.
3. **47** — volumen/páginas erróneos (1(4):233-243 → 1(1):40-43) y no es fuente específica de atletas.
4. **48** — páginas 129-135 → 129-133.
5. **56** — la URL corresponde a Oku et al., no a Manz & Woldt.
6. **57** — DOI inexistente; fuente correcta no localizada.
7. **58** — DOI inexistente; usar 10.1080/02640414.2011.585473.
8. **59** — DOI inexistente y autor erróneo ("Louard" → "Van Loon"); usar 10.1080/02640414.2011.619204.
9. **60** — la URL es un estudio de supermercados, no de colágeno/caldo de huesos.
10. **62** — DOI inexistente; fuente correcta no localizada.
11. **63** — páginas 115-122 → 115-119.
12. **70, 71** — DOI inexistente y artículo no localizado.
13. **72** — la URL apunta a Patton et al. (1982), no a Ericson.
14. **74** — la fuente es de ortesis plantares, no de asimetría/lesión.
15. **78** — HTTP 404 y DOI inexistente.
16. **79** — DOI inexistente.
17. **87** — dominio de origen ya no sirve el PDF.
