"""Figuras PNG (300 dpi) en figures/. Cada figura es una función registrada en FIGURAS;
la tarea 7b añade `mapa` y `practica_distribucion` registrándolas con @figura."""
import argparse
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

try:
    from comun import RAIZ, leer_especies
except ImportError:
    from python.comun import RAIZ, leer_especies

J = 10
GAMMA_ESTRELLA = 3
INSTANCIAS = ["1ha", "map_27"]
FIGURAS = {}


def figura(nombre, por_instancia=True):
    def deco(f):
        FIGURAS[nombre] = (f, por_instancia)
        return f
    return deco


def _guardar(fig, ruta):
    Path(ruta).parent.mkdir(parents=True, exist_ok=True)
    fig.tight_layout()
    fig.savefig(ruta, dpi=300)
    plt.close(fig)


def _nombres_cortos():
    return [n.split()[0][0] + ". " + n.split()[1][:6] for n in leer_especies()["especie"]]


@figura("C_por_especie")
def fig_C_por_especie(inst, origen, salida):
    comp = pd.read_csv(origen / "paso1_composicion.csv").set_index("instancia").loc[inst]
    D = comp[[f"D_{j}" for j in range(J)]].to_numpy(float)
    X = pd.read_csv(origen / f"escenarios_X_{inst}.csv")[[f"X_{j}" for j in range(J)]].to_numpy(float)
    C = D[None, :] - X
    fig, ax = plt.subplots(figsize=(8, 4.5))
    ax.boxplot([C[:, j] for j in range(J)], tick_labels=_nombres_cortos(), showfliers=True,
               flierprops={"markersize": 2, "alpha": 0.4})
    ax.axhline(0, color="#888888", lw=0.8, ls="--")
    ax.set_ylabel("C_j = D*_j - X_j (plantas a comprar)")
    ax.set_title(f"Requerimiento por especie en los escenarios ({inst})")
    plt.setp(ax.get_xticklabels(), rotation=40, ha="right")
    _guardar(fig, salida / f"C_por_especie_{inst}.png")


@figura("curva_compromiso")
def fig_curva(inst, origen, salida):
    r = pd.read_csv(origen / "paso1_resumen_gamma.csv")
    r = r[(r["instancia"] == inst) & (r["gamma"] <= 100)].sort_values("gamma")
    x, y = r["suma_theta_d"].to_numpy(), r["z_opt_media"].to_numpy()
    err = np.vstack([y - r["z_opt_min"], r["z_opt_max"] - y])
    fig, ax = plt.subplots(figsize=(7, 4.5))
    ax.errorbar(x, y, yerr=err, fmt="o-", color="#1b6ca8", capsize=3, lw=1.2)
    for xi, yi, g in zip(x, y, r["gamma"]):
        ax.annotate(f"{g:g}", (xi, yi), textcoords="offset points", xytext=(5, 5), fontsize=8)
    e = r[np.isclose(r["gamma"], GAMMA_ESTRELLA)]
    if len(e):
        ax.plot(e["suma_theta_d"], e["z_opt_media"], "*", color="#d95f02", ms=16,
                label=f"γ* = {GAMMA_ESTRELLA}", zorder=5)
        ax.legend()
    ax.set_xlabel("Σ θ_j · D_j (costo de mortalidad esperada)")
    ax.set_ylabel("Z_opt medio (min–máx entre semillas)")
    ax.set_title(f"Curva de compromiso competencia vs. costo ({inst})")
    _guardar(fig, salida / f"curva_compromiso_{inst}.png")


@figura("convergencia")
def fig_convergencia(inst, origen, salida):
    d = pd.read_csv(origen / f"convergencia_{inst}.csv")
    fig, ax = plt.subplots(figsize=(7, 4.2))
    ax.plot(d["iteracion"], d["actual"], color="#bbbbbb", lw=0.8, label="actual")
    ax.plot(d["iteracion"], d["mejor"], color="#1b6ca8", lw=1.6, label="mejor")
    ax.set_xlabel("iteración")
    ax.set_ylabel("objetivo")
    ax.set_title(f"Convergencia de la búsqueda tabú ({inst})")
    ax.legend()
    _guardar(fig, salida / f"convergencia_{inst}.png")


@figura("practica_compra", por_instancia=False)
def fig_practica_compra(origen, salida, instancias=None):
    d = pd.read_csv(origen / "practica_compra.csv")
    d = d[d["instancia"].isin(instancias or INSTANCIAS)]
    insts = list(dict.fromkeys(d["instancia"]))
    ops = ["a", "b", "c", "d"]
    ancho = 0.8 / len(insts)
    fig, axs = plt.subplots(1, 3, figsize=(13, 4.2))
    paneles = [("costo_ha", "costo por hectárea ($)"), ("faltante_medio", "faltante medio (plantas)"),
               ("supervivencia", "supervivencia esperada")]
    colores = ["#1b6ca8", "#d95f02", "#1b9e77", "#7570b3"]
    for ax, (col, etq) in zip(axs, paneles):
        for k, inst in enumerate(insts):
            s = d[d["instancia"] == inst].set_index("opcion").reindex(ops)
            ax.bar(np.arange(4) + k * ancho, s[col], ancho, label=inst, color=colores[k % 4])
        ax.set_xticks(np.arange(4) + ancho * (len(insts) - 1) / 2)
        ax.set_xticklabels(["a: 658/ha", "b: 524/ha", "c: ε=0", "d: ε=0.1"])
        ax.set_ylabel(etq)
        if col == "costo_ha":
            ax.axhline(12500, color="red", ls="--", label="presupuesto 12,500")
        if col == "supervivencia":
            ax.set_ylim(0.9, 0.93)
    axs[0].legend(fontsize=8)
    fig.suptitle("Práctica actual vs. modelo: compra de plantas")
    _guardar(fig, salida / "practica_compra.png")


def main(nombres=None, instancias=None, origen=None, salida=None):
    origen = Path(origen or RAIZ / "results")
    salida = Path(salida or RAIZ / "figures")
    for n in nombres or list(FIGURAS):
        f, por_inst = FIGURAS[n]
        if por_inst:
            for inst in instancias or INSTANCIAS:
                f(inst, origen, salida)
        else:
            f(origen, salida, instancias)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("figuras", nargs="*", help=f"alguna de {list(FIGURAS)}; vacío = todas")
    ap.add_argument("--instancias", nargs="*")
    a = ap.parse_args()
    main(a.figuras or None, a.instancias)
