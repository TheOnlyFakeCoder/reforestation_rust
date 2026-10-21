"""Tabla antes/después de la fase 4: compara los CSV versionados de dos commits y escribe results/cambios_fase4.md."""
import argparse
import io
import subprocess
from pathlib import Path

import numpy as np
import pandas as pd

try:
    from comun import RAIZ, leer_config
    import resumen
except ImportError:
    from python.comun import RAIZ, leer_config
    from python import resumen

ARCHIVOS = {
    "paso1_gamma_star": "paso1_gamma_star.csv",
    "paso1_composicion": "paso1_composicion.csv",
    "paso3_resumen": "paso3_resumen.csv",
    "paso2_compra": "paso2_compra.csv",
    "practica_distribucion": "practica_distribucion.csv",
}
NOTA_PUNTOS = ("En la fila `reducción vs serpentina iii`, `antes` y `después` son porcentajes y "
               "`cambio_pct` es la diferencia en puntos porcentuales (después − antes), no un "
               "cambio relativo.")
GAMMA_STAR = 3
EPSILON = 0.1
PI_FACTOR = 2


def leer_git(commit, ruta):
    """CSV de `git show <commit>:<ruta>` como DataFrame."""
    r = subprocess.run(["git", "show", f"{commit}:{ruta}"], cwd=RAIZ, capture_output=True, check=True)
    return pd.read_csv(io.BytesIO(r.stdout))


def _vec(f, pref):
    return "[" + ", ".join(f"{float(f[f'{pref}_{j}']):g}" for j in range(10)) + "]"


def _pct(a, d):
    return (d - a) / a * 100 if a else np.nan


def _fila(cifra, inst, a, d, pct=None):
    return {"cifra": cifra, "instancia": inst, "antes": a, "despues": d,
            "cambio_pct": _pct(a, d) if pct is None else pct}


def _una(df, inst):
    s = df[df["instancia"] == inst]
    return s.iloc[0] if len(s) else None


def tabla_cambios(antes, despues):
    filas = []
    for inst in ("1ha", "map_27", "map_21"):
        ga, gd = antes["paso1_gamma_star"], despues["paso1_gamma_star"]
        sa = ga[(ga["instancia"] == inst) & (ga["gamma"] == GAMMA_STAR)]
        sd = gd[(gd["instancia"] == inst) & (gd["gamma"] == GAMMA_STAR)]
        if len(sa) and len(sd):
            filas.append(_fila("Z_opt paso 1", inst, float(sa["z_opt"].mean()), float(sd["z_opt"].mean())))
        ca, cd = _una(antes["paso1_composicion"], inst), _una(despues["paso1_composicion"], inst)
        if ca is not None and cd is not None:
            va, vd = _vec(ca, "D"), _vec(cd, "D")
            filas.append(_fila("D* paso 1", inst, va, vd, 0.0 if va == vd else np.nan))
        pa, pd_ = _una(antes["paso3_resumen"], inst), _una(despues["paso3_resumen"], inst)
        if pa is not None and pd_ is not None:
            filas.append(_fila("Z media paso 3", inst, float(pa["Z_media"]), float(pd_["Z_media"])))
    for inst in ("1ha", "map_27"):
        def compra(df):
            s = df[(df["instancia"] == inst) & np.isclose(df["epsilon"], EPSILON) & (df["pi_factor"] == PI_FACTOR)]
            return s.iloc[0] if len(s) else None
        ka, kd = compra(antes["paso2_compra"]), compra(despues["paso2_compra"])
        if ka is not None and kd is not None:
            filas.append(_fila("costo paso 2", inst, float(ka["costo"]), float(kd["costo"])))
            va, vd = _vec(ka, "q"), _vec(kd, "q")
            filas.append(_fila("q paso 2", inst, va, vd, 0.0 if va == vd else np.nan))

        def reduccion(df):
            s = df[df["instancia"] == inst].set_index("variante")["Z"]
            if "iii_serpentina_existentes" not in s or "tabu_con_existentes" not in s:
                return None
            return 100 * (1 - s["tabu_con_existentes"] / s["iii_serpentina_existentes"])
        ra, rd = reduccion(antes["practica_distribucion"]), reduccion(despues["practica_distribucion"])
        if ra is not None and rd is not None:
            filas.append(_fila("reducción vs serpentina iii", inst, float(ra), float(rd),
                               pct=float(rd) - float(ra)))
    return pd.DataFrame(filas, columns=["cifra", "instancia", "antes", "despues", "cambio_pct"])


def _tabla_validacion(origen):
    ruta = Path(origen) / "validacion_exacta.csv"
    if not ruta.exists():
        return "No hay `validacion_exacta.csv`."
    v = pd.read_csv(ruta)
    v["optimo_probado"] = v["optimo_probado"].astype(bool)
    d = resumen.validacion_definitiva(resumen.brechas_vs_cota_valida(v))
    cols = [c for c in ("id", "modelo", "formulacion", "z_opt", "cota", "optimo_probado",
                        "z_ts_mejor", "z_ts_media", "brecha_mejor", "brecha_media") if c in d.columns]
    return resumen.tabla_md(d[cols])


def main(antes="be65ce0", despues="45c370f", salida="results/cambios_fase4.md"):
    a = {k: leer_git(antes, f"results/{f}") for k, f in ARCHIVOS.items()}
    d = {k: leer_git(despues, f"results/{f}") for k, f in ARCHIVOS.items()}
    tabu = leer_config()["tabu"]
    texto = "\n\n".join([
        f"# Cambios de la fase 4\n\nAntes: `{antes}`. Después: `{despues}`.",
        "## Cifras antes y después\n\n" + NOTA_PUNTOS + "\n\n" + resumen.tabla_md(tabla_cambios(a, d)),
        "## Validación exacta (filas definitivas)\n\n" + _tabla_validacion(RAIZ / "results"),
        "## Parámetros del tabú\n\n" + resumen.tabla_md(pd.DataFrame([tabu])),
    ]) + "\n"
    ruta = Path(salida)
    if not ruta.is_absolute():
        ruta = RAIZ / ruta
    ruta.write_text(texto, encoding="utf-8", newline="\n")
    return ruta


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--antes", default="be65ce0")
    ap.add_argument("--despues", default="45c370f")
    ap.add_argument("--salida", default="results/cambios_fase4.md")
    x = ap.parse_args()
    print(main(x.antes, x.despues, x.salida))
