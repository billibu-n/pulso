#!/usr/bin/env python3
"""Comprueba si el sistema tiene TODO lo que un documento necesita para compilar.

POR QUE EXISTE, y es una leccion de este proyecto: durante el 2026-10-13 se pidieron los
paquetes de LaTeX DE UNO EN UNO, segun iba fallando la compilacion. Fueron SEIS viajes a la
terminal del dueno, y tres de ellos pedian paquetes que el documento no usaba.

La regla que sale de ahi: cuando algo falla por una pieZA QUE FALTA, no se arregla la que se
vio: SE BUSCAN TODAS LAS QUE FALTAN, y se da la lista entera.

Uso:
    python3 herramientas/revisar-latex.py <doc.md> [<plantilla.tex>]

Sin argumentos, comprueba la plantilla del molde de documentos y el informe del cliente.
"""
import re, subprocess, sys, os, io

PLANTILLA = os.path.expanduser("~/projects/norma-documentos/moldes/plantilla-base.tex")
INFORME = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                       "informe", "documento.md")


def paquetes_de(texto):
    """Los paquetes que pide un .tex o un .md (via pandoc)."""
    encontrados = set()
    for m in re.finditer(r'\\usepackage(?:\[[^\]]*\])?\{([^}]+)\}', texto):
        for p in m.group(1).split(','):
            p = p.strip()
            if p:
                encontrados.add(p)
    return encontrados


def paquetes_de_pandoc(ruta_md):
    """Lo que pandoc AÑADE al preambulo, que no esta en el .md."""
    try:
        r = subprocess.run(["pandoc", ruta_md, "-o", "/dev/stdout", "--standalone"],
                           capture_output=True, text=True, timeout=120)
        return paquetes_de(r.stdout)
    except Exception:
        return set()


def falta(p):
    """¿Esta el paquete en el sistema? Se pregunta por el FICHERO, no por el nombre."""
    if p in ("inputenc", "fontenc", "babel"):  # estos no siempre son .sty
        return subprocess.run(["kpsewhich", p + ".ldf"], capture_output=True).returncode != 0 and \
               subprocess.run(["kpsewhich", p + ".sty"], capture_output=True).returncode != 0
    return subprocess.run(["kpsewhich", p + ".sty"], capture_output=True).returncode != 0


def pide_dentro(paquete):
    """¿Que pide por dentro un paquete ya instalado? Devuelve los nombres."""
    r = subprocess.run(["kpsewhich", paquete + ".sty"], capture_output=True, text=True)
    fichero = r.stdout.strip()
    if not fichero or not os.path.exists(fichero):
        return set()
    try:
        t = io.open(fichero, encoding='utf-8', errors='replace').read()
    except Exception:
        return set()
    dentro = set()
    for m in re.finditer(r'\\RequirePackage(?:\[[^]]*\])?\{([^}]+)\}', t):
        for x in m.group(1).split(','):
            x = x.strip()
            # `#1` y compañía son ARGUMENTOS de macro, no paquetes. Se descartan.
            if x and not x.startswith('#') and not x.startswith('@'):
                dentro.add(x)
    return dentro


def cerrar_dependencias(pedidos, tope=4):
    """Sigue lo que piden DENTRO los paquetes, hasta que no aparezca nada nuevo.

    POR QUE HACE FALTA: el 2026-10-13 el molde fallo por `tikzfill.image`, que NO lo pide
    el documento: lo pide `tcolorbox` desde dentro de un fichero suyo
    (`tcbskins.code.tex`). Mirar solo la lista de fuera no lo ve.
    """
    conocidos = set(pedidos)
    frontera = set(pedidos)
    for _ in range(tope):
        nuevos = set()
        for p in frontera:
            for q in pide_dentro(p):
                if q not in conocidos:
                    nuevos.add(q)
        if not nuevos:
            break
        conocidos |= nuevos
        frontera = nuevos
    return conocidos


def paquete_fedora(fichero):
    """A que paquete de Fedora pertenece un .sty que falta."""
    try:
        r = subprocess.run(["dnf", "provides", f"*/{fichero}.sty"],
                           capture_output=True, text=True, timeout=180)
        for m in re.finditer(r'texlive-[a-z0-9-]+-\d+', r.stdout):
            return m.group(0).rsplit('-', 1)[0]
    except Exception:
        pass
    return None


def main():
    md = sys.argv[1] if len(sys.argv) > 1 else INFORME
    plat = sys.argv[2] if len(sys.argv) > 2 else PLANTILLA

    pedidos = set()
    fuentes = []

    if os.path.exists(plat):
        pedidos |= paquetes_de(io.open(plat, encoding='utf-8', errors='replace').read())
        fuentes.append(f"plantilla {os.path.basename(plat)}")
    if os.path.exists(md):
        pedidos |= paquetes_de_pandoc(md)
        fuentes.append(f"documento {os.path.basename(md)}")

    directos = len(pedidos)
    # Y ahora lo que piden DENTRO esos paquetes: es donde se esconden los que faltan de verdad.
    pedidos = cerrar_dependencias(pedidos, tope=2)

    print(f"Paquetes revisados: {directos} directos, {len(pedidos)} contando los de dentro")
    print(f"Fuentes: {', '.join(fuentes)}\n")

    faltan = sorted(p for p in pedidos if falta(p))

    # Las DOS lecciones del 2026-10-13, aprendidas a golpes de compilacion fallida:
    #
    #  1. `#1` es un ARGUMENTO de macro, no un paquete.
    #  2. Los nombres que NO EXISTEN en los repositorios de Fedora suelen ser comprobaciones
    #     internas de LaTeX (`\@ifpackageloaded`, `\IfFileExists`), no dependencias de verdad.
    #     Se separan, para no hacer instalar lo que no hace falta.
    reales, dudosos = [], []
    for p in faltan:
        (dudosos if paquete_fedora(p) is None else reales).append(p)
    faltan = reales

    if not faltan:
        print("✅ TODO INSTALADO: el documento puede compilar.")
        return 0

    print(f"❌ FALTAN {len(faltan)}: {', '.join(faltan)}\n")
    print("Los paquetes de Fedora que los traen:\n")
    fedora = []
    for f in faltan:
        p = paquete_fedora(f)
        print(f"  {f:<16} -> {p or 'NO ENCONTRADO en los repos'}")
        if p:
            fedora.append(p)

    if fedora:
        print("\nLa orden completa, de una vez:\n")
        print("  sudo dnf install " + " ".join(sorted(set(fedora))))

    if dudosos:
        print(f"\n⚠  Y estos {len(dudosos)} NO están, pero NO se piden para instalar:")
        for p in dudosos:
            print(f"    {p}")
        print("    Motivo: no existen como paquete en los repositorios, así que lo más")
        print("    probable es que sean comprobaciones internas de LaTeX y no dependencias.")
        print("    Si la compilación vuelve a fallar por uno de ellos, entonces sí: se busca a mano.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
