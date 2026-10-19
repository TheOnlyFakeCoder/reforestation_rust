"""Genera results/RESUMEN.md a partir de los CSV/JSON de results/."""
import argparse
import json
from pathlib import Path

import numpy as np
import pandas as pd

try:
    from comun import RAIZ, leer_config
except ImportError:
    from python.comun import RAIZ, leer_config

INSTANCIAS = ["1ha", "map_27"]
FIGURAS_LISTA = [
    ("hist_rho.png", "Histograma de la densidad ρ con las densidades ajustadas"),
    ("C_por_especie_1ha.png", "Requerimiento C_j por especie, 1 ha"),
    ("C_por_especie_map_27.png", "Requerimiento C_j por especie, P1 (map_27)"),
    ("curva_compromiso_1ha.png", "Curva de compromiso competencia vs. costo, 1 ha"),
    ("curva_compromiso_map_27.png", "Curva de compromiso competencia vs. costo, P1"),
    ("convergencia_1ha.png", "Convergencia del tabú del paso 1, 1 ha"),
    ("convergencia_map_27.png", "Convergencia del tabú del paso 1, P1"),
    ("mapa_1ha.png", "Mapa socia vs. tabú (paso 3, ω = 0), 1 ha"),
    ("mapa_map_27.png", "Mapa socia vs. tabú (paso 3, ω = 0), P1"),
    ("practica_compra.png", "Práctica actual vs. modelo: compra de plantas"),
    ("practica_distribucion.png", "Práctica actual vs. tabú: distribución"),
]


def _fmt(v):
    if isinstance(v, (bool, np.bool_)):
        return str(bool(v))
    if isinstance(v, float):
        if v != v:
            return ""
        return format(round(v, 4), "f").rstrip("0").rstrip(".")
    if v is None:
        return ""
    return str(v)


def tabla_md(df):
    cols = list(df.columns)
    lineas = ["| " + " | ".join(str(c) for c in cols) + " |",
              "| " + " | ".join("---" for _ in cols) + " |"]
    for fila in df.itertuples(index=False):
        lineas.append("| " + " | ".join(_fmt(v) for v in fila) + " |")
    return "\n".join(lineas)


def _csv(origen, nombre):
    return pd.read_csv(origen / nombre)


def _malla(o):
    r = _csv(o, "malla_resumen.csv")
    an = _csv(o, "malla_anillos.csv")
    ren = {"n_pos": "\\|V\\|", "round_658A": "round(658·A)"}
    sel = r[r["instancia"].isin(INSTANCIAS)]
    cols = ["instancia", "area_ha", "n_pos", "round_658A", "excedente", "d_tot"]
    pa = an[an["instancia"].isin(INSTANCIAS)].pivot(index="anillo", columns="instancia",
                                                    values="pares").reset_index()
    pa = pa[["anillo"] + [i for i in INSTANCIAS if i in pa.columns]]
    cols3 = ["instancia", "area_ha", "columnas", "filas", "n_pos", "round_658A", "excedente",
             "d_tot", "pares_anillo_1"]
    return "\n\n".join([
        "### 1 ha y P1 (map_27)\n\n" + tabla_md(sel[cols].rename(columns=ren)),
        "Pares por anillo:\n\n" + tabla_md(pa),
        f"### Las {len(r)} instancias\n\n" + tabla_md(r[cols3].rename(columns=ren)),
    ])


def _ajuste(o):
    r = _csv(o, "ajuste_resumen.csv")
    dens = _csv(o, "ajuste_densidad.csv")
    mc = json.loads((o / "mc_params.json").read_text(encoding="utf-8"))
    p = pd.DataFrame({"especie": range(len(mc["p_hat"])), "p_hat": mc["p_hat"]})
    return (tabla_md(r) + "\n\nAjuste de la densidad ρ:\n\n" + tabla_md(dens)
            + f"\n\nParámetros de Monte Carlo (`mc_params.json`): μ = {mc['mu']:.4f}, "
            f"σ = {mc['sigma']:.4f}.\n\n" + tabla_md(p))


def _paso1(o, gamma_star):
    g = _csv(o, "paso1_resumen_gamma.csv")
    partes = [f"γ* = {gamma_star:g}, elegido por el usuario en el codo de la curva de compromiso."]
    for inst in INSTANCIAS:
        s = g[g["instancia"] == inst][["gamma", "suma_theta_d", "suma_c_d", "z_opt_media", "z_opt_min",
                                       "z_opt_max", "z_opt_por_planta_media"]]
        partes.append(f"Barrido de γ, {inst}:\n\n" + tabla_md(s))
    c = _csv(o, "paso1_composicion.csv")
    cols = ["instancia", "d_tot"] + [f"D_{j}" for j in range(10)]
    partes.append("D* por instancia (γ*):\n\n" + tabla_md(c[cols]))
    return "\n\n".join(partes)


def _mc(o):
    e = _csv(o, "mc_resumen_especies.csv")
    e = e[e["instancia"].isin(INSTANCIAS)][["instancia", "composicion", "especie", "D", "E_C", "sd_C",
                                            "P90", "P95", "N_necesario"]]
    c = _csv(o, "mc_resumen_costo.csv")
    return ("Por especie (1 ha y P1):\n\n" + tabla_md(e)
            + f"\n\nCosto (las {c['instancia'].nunique()} instancias):\n\n"
            + tabla_md(c.drop(columns=["map_id"])))


def _paso2(o, cfg):
    p = _csv(o, "paso2_compra.csv")
    p = p.assign(q_total=p[[f"q_{j}" for j in range(10)]].sum(axis=1))
    eps, pi = cfg.get("epsilon", 0.1), cfg.get("pi_factor", 2)
    mism_eps = p["epsilon"].round(6) == round(eps, 6)
    base = p[mism_eps & (p["pi_factor"] == pi)]
    cols = ["instancia", "q_total", "costo", "presupuesto", "cabe", "faltante_esperado",
            "sobrante_esperado"]
    comp = p[mism_eps & p["instancia"].isin(INSTANCIAS)]
    return "\n\n".join([
        f"ε = {eps:g}, π = {pi:g}·c ({len(base)} instancias):\n\n" + tabla_md(base[cols]),
        "Comparación de π (1 ha y P1):\n\n" + tabla_md(
            comp[["instancia", "pi_factor"] + cols[1:]]),
    ])


def _paso3(o):
    r = _csv(o, "paso3_resumen.csv")
    tot = int(r["casos_presupuesto_corto"].sum())
    n = int(r["n"].sum())
    nota = (f"Los escenarios caen en `presupuesto_corto` en {tot} de {n} casos: con q cercano a la "
            "mediana (π = 2c) la compra deja unas 12 posiciones vacías por hectárea. Es un hallazgo "
            "del modelo, no un error.")
    return nota + "\n\n" + tabla_md(r.drop(columns=["map_id"]))


def _practica(o):
    c = _csv(o, "practica_compra.csv")
    c = c[c["instancia"].isin(INSTANCIAS)].drop(columns=["map_id", "escenarios"])
    d = _csv(o, "practica_distribucion.csv")
    d = d[d["instancia"].isin(INSTANCIAS)]
    m = _csv(o, "markov.csv")
    m = m[m["instancia"].isin(INSTANCIAS)].groupby(["instancia", "composicion"], as_index=False)[
        ["plantas", "vivas_1ano", "vivas_2anos", "reposicion_esperada"]].sum()
    return ("Compra (a nominal 658/ha, b nominal 524/ha, c paso 2 ε = 0, d paso 2 ε = 0.1):\n\n"
            + tabla_md(c) + "\n\nDistribución:\n\n" + tabla_md(d)
            + "\n\nMarkov (totales por instancia y composición):\n\n" + tabla_md(m))


def _figuras():
    return "\n".join(f"- [{t}](../figures/{f})" for f, t in FIGURAS_LISTA)


def generar(origen):
    origen = Path(origen)
    cfg = leer_config()
    return "\n\n".join([
        "# Resumen de resultados: reforestación CONAFOR (Entregable 2)",
        "Generado por `python/resumen.py` a partir de los CSV y JSON de `results/`.",
        "## Malla\n\n" + _malla(origen),
        "## Ajuste de distribuciones\n\n" + _ajuste(origen),
        "## Paso 1: composición y competencia\n\n" + _paso1(origen, cfg.get("gamma_star", 3)),
        "## Monte Carlo\n\n" + _mc(origen),
        "## Paso 2: compra\n\n" + _paso2(origen, cfg),
        "## Paso 3: ajuste al polígono real\n\n" + _paso3(origen),
        "## Práctica actual\n\n" + _practica(origen),
        "## Figuras\n\n" + _figuras(),
    ]) + "\n"


def main(origen=None, salida=None):
    origen = Path(origen or RAIZ / "results")
    salida = Path(salida or origen / "RESUMEN.md")
    salida.write_text(generar(origen), encoding="utf-8", newline="\n")
    return salida


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--origen")
    ap.add_argument("--salida")
    a = ap.parse_args()
    print(main(a.origen, a.salida))
