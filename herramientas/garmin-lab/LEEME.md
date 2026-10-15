# Instrumentos de Garmin (la vía del navegador)

**Escritos el 2026-10-15.** Son los instrumentos con los que se **midió** la vía de sincronización
con Garmin, y con los que se traen los datos cuando la aplicación todavía no lo hace. El resultado
medido está en `docs/SINCRONIZACION-garmin-2026-10-15.md`.

**Por qué existen**: los flujos programáticos de acceso reciben 403/429 de Garmin y de Cloudflare, y
el navegador oficial entra sin problemas. La conclusión medida es que **hace falta un motor de
navegador** — el acceso *y* las llamadas a los datos. Estos instrumentos son la prueba y la
herramienta provisional.

## Dónde corren

En el **equipo Kali** (es la única máquina del taller con navegador y con `DISPLAY`), en
`~/garmin-lab/`, con su propio entorno:

```bash
python3 -m venv ~/garmin-lab/venv
~/garmin-lab/venv/bin/pip install playwright curl_cffi
~/garmin-lab/venv/bin/playwright install chromium
```

No escriben **nada** dentro del repositorio: todo va a `~/garmin-lab/` (datos, sesión, capturas).

## En qué orden se usan

| # | instrumento | qué hace | ¿gasta ventana? |
|---|---|---|---|
| 1 | `sonda.py` | Sonda **sin credenciales**: navegador, página oficial, campos del formulario, cookies y huellas. | no |
| 2 | `portal.py` | Inspecciona el portal y compara lo que recibe un cliente programático. | no |
| 3 | `entrar.py` | Abre el navegador y **la persona entra a mano** (correo, contraseña y reCAPTCHA **en la página de Garmin**). Con `GARMIN_AUTO=1` lo intenta él (solo sirve si no hay reCAPTCHA). | **sí** |
| 4 | `sesion.py` | Saca el material de sesión a un fichero 600 y comprueba que sigue dentro. | no |
| 5 | `mirar.py` | Apunta el tráfico real de la app (`/gc-api/…`) y prueba un `fetch` interno. | no |
| 6 | `dentro.py` | **Separa cabeceras de transporte.** Es el que da el veredicto: por código, 403 de Cloudflare. | no |
| 7 | `datos.py` | **Trae los datos** (sueño, estrés, training status, pulso en reposo, resumen del día…). | no |

Los pasos 4 a 7 se apoyan en el **perfil del navegador**, que conserva la sesión: se entra **una vez**
y no hay que repetir el reCAPTCHA.

## Trato de los secretos (no negociable)

- Las credenciales se teclean **en la página de Garmin**, nunca en una terminal ni en un chat.
- Los ficheros con material de sesión (`sesion-navegador.json`, `cabeceras-gcapi.json`, `token.json`,
  `ticket.txt`) se escriben con **permisos 600** y **no se imprimen nunca**: los instrumentos enseñan
  códigos HTTP, tamaños y **nombres** de cookies o cabeceras, jamás sus valores.
- Nada de esto se copia al repositorio: los datos son de salud y van a `datos/`, que está ignorado.

## Aviso

`entrar.py` **no** pulsa «Sign In» por su cuenta si falta la contraseña ni intenta resolver el
reCAPTCHA: en modo manual solo mira. Los reintentos contra el acceso se evitan a propósito — cada
intento fallido agrava la desconfianza de Garmin y alarga la ventana de espera.
