# ESTADO DEL ARTE: Análisis de entrenamiento para ciclismo

> **REVISADO FILA A FILA el 2026-10-14**, abriendo las URLs de verdad y comprobando cada DOI contra la
> API de Crossref y PubMed. Los dos informes de la revisión están en
> [ESTADO-DEL-ARTE-revision-1-2.md](ESTADO-DEL-ARTE-revision-1-2.md) y
> [ESTADO-DEL-ARTE-revision-3-6.md](ESTADO-DEL-ARTE-revision-3-6.md).
>
> **TRES COSAS QUE SALIERON DE LA REVISIÓN, y que valen más que el documento:**
> 1. **Una fuente falsa es peor que una ausente.** Se encontraron DOI que NO existen, un autor
>    inventado y artículos cuyo título no consta en ninguna base. Se corrigen, y lo que no se pudo
>    sostener se deja **SIN FUENTE LOCALIZADA** antes que rellenarlo con una invención.
> 2. **Un 403 NO es una URL muerta.** 19 de las 25 URLs únicas bloquean a los clientes automáticos
>    (Cloudflare). La fuente EXISTE; lo que no se pudo es leerla desde aquí, y por eso las citas
>    numéricas de esas filas quedan **SIN VERIFICAR**.
> 3. **Un dato no se identifica por su nombre, sino por su definición** — y una cita tampoco: un DOI
>    copiado a mano suele estar a un dígito del bueno, y ese dígito es la diferencia entre una fuente y
>    una mentira.
>
> **CÓMO LEER LA ÚLTIMA COLUMNA:** una URL es la fuente consultable; **SIN FUENTE LOCALIZADA** es una
> fila que se deja a propósito sin fuente; **(libro, sin URL)** es una obra real que no tiene dirección
> en línea; **(403)** es una URL válida que bloquea a los programas.

## 1. Zonas de potencia en ciclismo

| qué dice | valor o regla | fuente | dirección |
|----------|---------------|--------|-----------|
| Número de zonas estándar según Coggan | 7 zonas | **Allen, H., & Coggan, A.R.** *Training and Racing with a Power Meter* (1.ª ed. 2006). La fila citaba "Coggan 2001", que no es la edición. | (libro, sin URL). La tabla de Coggan en TrainingPeaks confirma las 7 zonas: https://www.trainingpeaks.com/blog/power-training-levels/ (VERIFICADA) |
| Corte de Zona 1 (Activa recuperación) | <55% FTP | Allen & Coggan (2006) | (libro, sin URL). **VERIFICADA PARCIAL**: la tabla web rotula "1 Active Recovery" pero no publica su porcentaje |
| Corte de Zona 2 (Resistencia) | 56-75% FTP | Allen & Coggan (2006) | https://www.trainingpeaks.com/blog/power-training-levels/ (VERIFICADA) |
| Corte de Zona 3 (Tempo) | 76-90% FTP | Allen & Coggan (2006) | https://www.trainingpeaks.com/blog/power-training-levels/ (VERIFICADA) |
| Corte de Zona 4 (Umbral de lactato) | 91-105% FTP | Allen & Coggan (2006) | https://www.trainingpeaks.com/blog/power-training-levels/ (VERIFICADA) |
| Corte de Zona 5 (Capacidad aeróbica) | 106-120% FTP | Allen & Coggan (2006) | https://www.trainingpeaks.com/blog/power-training-levels/ (VERIFICADA) |
| Corte de Zona 6 (Capacidad anaeróbica) | **>121% FTP** (el tope 150% NO consta en la fuente web) | Allen & Coggan (2006) | https://www.trainingpeaks.com/blog/power-training-levels/ (VERIFICADA PARCIAL) |
| Corte de Zona 7 (Potencia neuromuscular) | **Maximal** (la fuente no publica un corte ">150%") | Allen & Coggan (2006) | https://www.trainingpeaks.com/blog/power-training-levels/ (VERIFICADA PARCIAL) |
| Sistema alternativo de 5 zonas (USA Cycling) | 5 zonas con cortes diferentes | USA Cycling, material de formación de entrenadores | **SIN FUENTE LOCALIZADA**: no se halló publicación con los cortes; el Rider Toolkit usa Z2/Z3 pero no define el sistema |
| Escala de Borg para RPE | 6-20 puntos, donde 6=sin esfuerzo y 20=esfuerzo máximo | Borg, G. (1982). *Psychophysical bases of perceived exertion*. MSSE 14(5):377-381. **PMID 7154893** | https://journals.lww.com/acsm-msse/Abstract/1982/10000/Psychophysical_bases_of_perceived_exertion.1.aspx (403 a programas; el artículo existe y el PMID lo confirma) |
| Valor RPE 9 | Esfuerzo muy ligero | Borg, G. (1982). PMID 7154893 | misma URL (403): **la etiqueta no se pudo citar textualmente** |
| Valor RPE 13 | Esfuerzo somewhat duro | Borg, G. (1982). PMID 7154893 | misma URL (403): la etiqueta no se pudo citar textualmente |
| Valor RPE 17 | Esfuerzo muy duro | Borg, G. (1982). PMID 7154893 | misma URL (403): la etiqueta no se pudo citar textualmente |
| Potencia crítica (CP) y W' | CP = potencia sostenible teóricamente sin fatiga; W' = trabajo finito por encima de CP | Morton, R.H. (2006). *The critical power and related whole-body bioenergetic models*. EJAP 96(4):339-354 | **URL CORREGIDA**: el DOI de la fila (10.1007/s00421-005-0065-8) NO existe. El bueno es **10.1007/s00421-005-0088-2** (comprobado en Crossref): https://link.springer.com/article/10.1007/s00421-005-0088-2 |
| Relación CP/FTP | CP típicamente corresponde al 90-95% del FTP | **SIN FUENTE: la cita que había NO EXISTE.** Vanhatalo et al. (2016), DOI 10.1080/17461391.2015.1067088, da 404 en Crossref y su título no consta. Vanhatalo, Jones & Burnley (2011), IJSPP 6(1):128-136, DOI 10.1123/ijspp.6.1.128, define la CP pero **NO** afirma ese 90-95% | **SIN FUENTE LOCALIZADA**. La afirmación queda retirada hasta tener una fuente que la sostenga |

## 2. Carga de entrenamiento

| qué dice | valor o regla | fuente | dirección |
|----------|---------------|--------|-----------|
| Fórmula TSS (Training Stress Score) | TSS = (segundos × NP × IF) / (FTP × 3600) × 100 | Coggan, A.R. (2005). *A Power Primer* (material de curso, sin URL) | https://www.trainingpeaks.com/learn/articles/estimating-training-stress-score-tss/ (VERIFICADA) |
| Fórmula NP (Normalized Power) | NP = media de las potencias de ventana de 30 s **elevadas a la 4.ª potencia**, y **raíz 4.ª** de esa media. (La fila lo tenía invertido.) | Coggan, A.R. (2005) | https://www.trainerroad.com/blog/normalized-power-what-it-is-and-how-to-use-it/ (fuente secundaria) |
| Fórmula IF (Intensity Factor) | IF = NP / FTP | Coggan, A.R. (2005) | https://www.trainingpeaks.com/learn/articles/normalized-power-intensity-factor-training-stress/ (VERIFICADA) |
| Cálculo CTL (Carga crónica) | CTL = CTL_ayer + (TSS_hoy − CTL_ayer) / 42 días | Coggan, A.R. (2005) | https://www.trainingpeaks.com/learn/articles/what-is-the-performance-management-chart/ (VERIFICADA: "last 42 days") |
| Cálculo ATL (Carga aguda) | ATL = ATL_ayer + (TSS_hoy − ATL_ayer) / 7 días | Coggan, A.R. (2005) | misma URL (VERIFICADA: "past seven days") |
| Cálculo TSB (Balance) | TSB = CTL − ATL | Coggan, A.R. (2005) | misma URL (VERIFICADA) |
| Umbral de TSS "peligroso" publicado | No existe umbral universalmente acordado | TrainingPeaks llama a su escala "approximate guide"; la cita que había (Mujika & Padilla, *Science and Cycling*, 2001) **no consta en Crossref** | **SIN FUENTE LOCALIZADA** para la cita original; la idea se sostiene en https://www.trainingpeaks.com/learn/articles/estimating-training-stress-score-tss/ |
| Fórmula ACWR | ACWR = ATL / CTL | Gabbett, T.J. (2016). BJSM 50(5):273-280. **DOI 10.1136/bjsports-2015-095788**, PMID 26758673 | https://bjsm.bmj.com/content/50/5/273 (403 a programas; el DOI es válido) |
| Validación de ACWR en ciclismo | evidencia mixta en prevención de lesiones | **SIN FUENTE: la cita que había (Williams et al., 2017, IJSPP 12(6):776-782) NO EXISTE con ese título.** Para la crítica del ACWR se propone Impellizzeri et al. (2020), IJSPP 15(6), PMID 32502973, **pendiente de comprobar** | **SIN FUENTE LOCALIZADA** |

## 3. Recuperación y sus señales medibles

| qué dice | valor o regla | fuente | dirección |
|----------|---------------|--------|-----------|
| Deriva cardíaca definida | Incremento progresivo de la FC a potencia constante | La cita de la fila (Coyle et al., 1992, ESSR 20:185-216) **no corresponde**; la revisión propone Coyle & González-Alonso (2001), ESSR 29(2):88-92, **pendiente de comprobar** | **SIN URL VERIFICABLE** (403) |
| Cálculo de deriva cardíaca | % = ((FC_final − FC_inicial) / FC_inicial) × 100 | misma situación | **SIN URL VERIFICABLE** (403) |
| Umbral significativo de deriva | >5% indicaría deshidratación o fatiga | misma situación | **SIN FUENTE VERIFICADA**: el umbral numérico no consta en la fuente accesible |
| HRV (RMSSD) definido | Raíz de la media de los cuadrados de las diferencias entre intervalos R-R | Malik, M., et al. (1996). EHJ 17(3):354-381. **DOI 10.1093/oxfordjournals.eurheartj.a014868** | https://academic.oup.com/eurheartj/article/17/3/354/213034 (403; DOI válido) |
| HRV durante ejercicio no comparable a tablas | El simpático domina y la parasimpática se retira | Akselrod, S., et al. (1981). Science 213(4504):220-222 | **DOI CORREGIDO**: el de la fila (10.1126/science.7198445) NO existe; el bueno es **10.1126/science.6166045** (comprobado en Crossref): https://www.science.org/doi/10.1126/science.6166045 |
| Variación significativa de pulso en reposo | >5 bpm sostenido sugiere falta de recuperación | Meeusen, R., et al. (2013). MSSE 45(1):186-205. **DOI 10.1249/MSS.0b013e318279a10a** | https://journals.lww.com/acsm-msse/Abstract/2013/01000/Prevention,_diagnosis,_and_treatment_of_the_Overtraining_Syndrome_.1.aspx (403; DOI válido) |
| Sueño recomendado para atletas | 7-9 horas con proporción de profundo y REM | Hirshkowitz, M., et al. (2015). Sleep Health **1(1):40-43** (la fila decía 1(4):233-243) | https://www.sleephealthjournal.org/article/S2352-7218(15)00094-5/abstract — **NO RESPALDA "atletas"**: la recomendación es de **población general** |
| Sueño y lesiones | <6 horas aumenta el riesgo | Milewski, M.D., et al. (2014). J Pediatr Orthop 34(2):**129-133** (la fila decía 135) | https://journals.lww.com/jpo/Abstract/2014/03000/Chronic_lack_of_sleep_is_associated_with_increased_sports_injuries_in.1.aspx (403) |

## 4. Nutrición e hidratación en deporte de resistencia

| qué dice | valor o regla | fuente | dirección |
|----------|---------------|--------|-----------|
| Líquido en ejercicio prolongado | 400-800 ml por hora, individualizado | ACSM (2007). MSSE 39(2):377-390. **DOI 10.1249/mss.0b013e31802ca597** | https://journals.lww.com/acsm-msse/Abstract/2007/02000/ACSM_Position_Stand__Exercise_and_Fluid_Replacement.1.aspx (403; DOI válido) |
| Sodio en ejercicio prolongado | 300-600 mg por hora | ACSM (2007) | misma URL (403) |
| Porcentaje de agua corporal de referencia | ~60% del peso en hombres adultos | **SIN FUENTE: nature.com/articles/1601666 es Oku et al. (2003), NO Manz & Woldt** | **SIN FUENTE LOCALIZADA** |
| Carbohidratos en ejercicio | 30-60 g/h (1-2,5 h); hasta 90 g/h (>2,5 h) | Jeukendrup, A.E., & Killer, S.C. (2010). EJSS 10(3):169-176 | **SIN FUENTE**: el DOI de la fila (10.1080/17461391.2010.489274) **no está registrado** |
| Carga de carbohidratos previa | 8-12 g/kg/día durante 36-48 h | Burke, L.M., et al. (2011). JSS 29(S1):S17-S27 | **DOI CORREGIDO**: 10.1080/02640414.2011.585473 (el de la fila, ...588425, NO existe): https://www.tandfonline.com/doi/abs/10.1080/02640414.2011.585473 |
| Proteína para deportistas | 1,2-2,0 g/kg/día | Phillips, S.M., & **Van Loon, L.J.C.** (2011). JSS 29(S1):S29-S38 (la fila decía "Louard", autor que no existe) | **DOI CORREGIDO**: 10.1080/02640414.2011.619204 (el de la fila, ...619241, NO existe): https://www.tandfonline.com/doi/abs/10.1080/02640414.2011.619204 |
| Evidencia para caldo de huesos | No hay evidencia sólida de beneficio superior a otras proteínas | **SIN FUENTE: el artículo citado (Nutrients 12(6):1692) es un estudio de SUPERMERCADOS, no de colágeno** | **SIN FUENTE LOCALIZADA** |
| Evidencia para electrolitos caseros | Equivalentes a soluciones comerciales con sodio y potasio | Hew-Butler, T., et al. (2015). CJSM 25(4):303-320 | https://journals.lww.com/cjsm/Abstract/2015/07000/Statement_of_the_3rd_International_Exercise-Associated_Hyponatremia_Consensus.1.aspx (403; la cita apunta bien) |
| Evidencia para CoQ10 | Inconsistente; beneficio mínimo en algunos estudios | López-Lluch, G., et al. (2019). ARS 31(14):1056-1085 | **SIN FUENTE**: el DOI de la fila (10.1089/ars.2018.7593) **no está registrado** |
| Evidencia para Omega-3 | Puede reducir inflamación y dolor muscular tardío | Tartibian, B., et al. (2009). CJSM 19(2):**115-119** (la fila decía 115-122) | https://journals.lww.com/cjsm/Abstract/2009/03000/The_effects_of_ingestion_of_omega-3_fatty_acids_on_severe_muscle_soreness.1.aspx (403) |
| Evidencia para calcio/vitamina D | Importante para la salud ósea; evidencia limitada para recuperación | Larson-Meyer, D.E., & Willis, K.S. (2010). CSMR 9(4):220-226 | https://journals.lww.com/csmr/Abstract/2010/07000/Vitamin_D_and_athletes.1.aspx (403) |

## 5. Mecánica de pedaleo y lesión

| qué dice | valor o regla | fuente | dirección |
|----------|---------------|--------|-----------|
| Fase de pedaleo (power phase) definida | Porcentaje del ciclo en que se aplica fuerza positiva | **SIN FUENTE**: el DOI de la fila (10.1080/02640414.2004.057752) **no existe** | **SIN FUENTE LOCALIZADA** |
| Unidad de medida de fase de pedaleo | % del ciclo completo (0-360 grados) | misma situación | **SIN FUENTE LOCALIZADA** |
| Valor normal de fase de pedaleo | 60-80% en ciclistas entrenados | **SIN FUENTE**: 10.1007/BF00428971 apunta a Patton et al. (1982), no a Ericson | **SIN FUENTE LOCALIZADA** |
| Asimetría entre piernas normal | 0-5% de diferencia en potencia | Paterson, D.J., et al. (1996). MSSE 28(11):1343-1350 | https://journals.lww.com/acsm-msse/Abstract/1996/11000/Bilateral_leg_muscle_activation_during_cycling_in_able_bodied_and_amputee_athletes.1.aspx (403) — **cita no localizada en Crossref/PubMed: SIN VERIFICAR** |
| Umbral de asimetría relevante | >10% sostenido indica posible desequilibrio | **SIN FUENTE: la cita (Barton et al., 2013, BJSM 47(3):156-163) es un metaanálisis de ORTESIS PLANTARES, no de asimetría** | **SIN FUENTE LOCALIZADA** |
| PCO (Platform Center Offset) definido | Distancia horizontal entre los tacos y el centro de presión | Hansen, E.A., et al. (2003). MSSE 35(5):805-811 | https://journals.lww.com/acsm-msse/Abstract/2003/05000/Power-cycling_relationship_at_different_pedaling_rates_and_pedal_configurations.1.aspx (403; el título no se localizó en Crossref: **SIN VERIFICAR**) |
| Unidad de medida de PCO | Milímetros (mm), positivo hacia el exterior | Hansen, E.A., et al. (2003) | misma URL (403) |
| Valor típico de PCO en ruta | 0-10 mm hacia el exterior | Hansen, E.A., et al. (2003) | misma URL (403) |
| Cadencia óptima según evidencia | 80-100 rpm en condiciones aeróbicas moderadas | Pocock, S., et al. (2011). EJAP 111(5):933-942 | **SIN FUENTE**: HTTP 404 y el DOI 10.1007/s00421-010-1696-2 **no está registrado** |
| Naturaleza discutida de la cadencia óptima | No hay consenso universal | Rosa, G., et al. (2017). EJSS 17(4):433-440 | **SIN FUENTE**: el DOI 10.1080/17461391.2016.1258921 **no está registrado** |

## 6. Arquitectura de software (para el programador)

| qué dice | valor o regla | fuente | dirección |
|----------|---------------|--------|-----------|
| Arquitectura hexagonal (puertos y adaptadores) | Alistair Cockburn, 2005 | Cockburn, A. (2005). *Hexagonal Architecture (Ports & Adapters)*, HaT Technical Report 2005.02, 4 de septiembre de 2005 | https://alistair.cockburn.us/hexagonal-architecture/ (**VERIFICADA**, HTTP 200) |
| Regla central de arquitectura hexagonal | El código de dentro no debe filtrarse hacia fuera | Cockburn, A. (2005) | misma URL (**VERIFICADA**): "As events arrive from the outside world at a port..." |
| Inversión de dependencias por Robert Martin | Los módulos de alto nivel no deben depender de los de bajo nivel | Martin, R.C. (2000). *Design Principles and Design Patterns* | **URL MUERTA**: objectmentor.com redirige hoy a un dominio aparcado. Alternativa viva: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html |
| Límites (boundaries) en arquitectura limpia | Definen dónde termina la responsabilidad de un componente | Martin, R.C. (2012). *Clean Architecture*. Prentice Hall | (libro, sin URL) — **SIN FUENTE LOCALIZADA** |
| Aporte de DDD para separar piezas | El modelo de dominio complejo se separa en capas | Evans, E. (2003). *Domain-Driven Design*. Addison-Wesley | (libro, sin URL) — **SIN FUENTE LOCALIZADA** |
| Regla de la dependencia | Las dependencias apuntan hacia adentro | Martin, R.C. (2012). *Clean Architecture* | (libro, sin URL). La afirmación está literalmente en https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html: "source code dependencies can only point inwards" |
