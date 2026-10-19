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


@figura("mapa")
def fig_mapa(inst, origen, salida):
    """Práctica de la socia (variante iii) contra el tabú del paso 3 (ω = 0) sobre la malla.
    En los patrones especie = -1 es posición vacía y 0..9 es la especie."""
    rutas = [origen / f"practica_patron_{inst}_iii.csv", origen / f"paso3_patron_{inst}_omega0.csv"]
    faltan = [r.name for r in rutas if not r.exists()]
    if faltan:
        print(f"aviso: mapa_{inst} omitido, falta {', '.join(faltan)}")
        return
    pr = pd.read_csv(origen / "practica_distribucion.csv")
    z_soc = pr[(pr["instancia"] == inst) & (pr["variante"] == "iii_serpentina_existentes")]["Z"]
    p3 = pd.read_csv(origen / "paso3_escenarios.csv")
    z_tabu = p3[(p3["instancia"] == inst) & (p3["omega"] == 0)]["Z"]
    titulos = ["Práctica de la socia (serpentina)", "Tabú, paso 3 (ω = 0)"]
    for k, z in enumerate([z_soc, z_tabu]):
        if len(z):
            titulos[k] += f"\nZ = {z.iloc[0]:.1f}"
    cmap = plt.get_cmap("tab10")
    nombres = _nombres_cortos()
    fig, axs = plt.subplots(1, 2, figsize=(14, 8))
    for ax, ruta, titulo in zip(axs, rutas, titulos):
        d = pd.read_csv(ruta)
        vac = d[d["especie"] < 0]
        ax.scatter(vac["x"], vac["y"], s=18, facecolors="none", edgecolors="#bbbbbb", lw=0.6)
        for j in range(J):
            sj = d[d["especie"] == j]
            nuevas = sj[sj["existente"] == 0]
            exis = sj[sj["existente"] == 1]
            ax.scatter(nuevas["x"], nuevas["y"], s=75, color=cmap(j), lw=0)
            ax.scatter(exis["x"], exis["y"], s=75, color=cmap(j), edgecolors="black", lw=1.6)
        ax.set_aspect("equal")
        ax.invert_yaxis()
        ax.set_xlabel("x (m)")
        ax.set_ylabel("y (m)")
        ax.set_title(titulo)
    asas = [plt.Line2D([], [], marker="o", ls="", color=cmap(j), label=nombres[j]) for j in range(J)]
    asas += [plt.Line2D([], [], marker="o", ls="", mfc="#999999", mec="black", mew=1.6, label="existente"),
             plt.Line2D([], [], marker="o", ls="", mfc="none", mec="#bbbbbb", label="vacía")]
    fig.suptitle(f"Distribución de especies en {inst}", fontsize=14)
    fig.tight_layout(rect=(0, 0.12, 1, 0.95))
    fig.legend(handles=asas, loc="lower center", ncol=6, fontsize=9, frameon=False,
               bbox_to_anchor=(0.5, 0.0))
    Path(salida).mkdir(parents=True, exist_ok=True)
    fig.savefig(Path(salida) / f"mapa_{inst}.png", dpi=300)
    plt.close(fig)


@figura("practica_distribucion", por_instancia=False)
def fig_practica_distribucion(origen, salida, instancias=None):
    d = pd.read_csv(origen / "practica_distribucion.csv")
    d = d[d["instancia"].isin(instancias or INSTANCIAS)]
    insts = list(dict.fromkeys(d["instancia"]))
    variantes = ["i_serpentina", "ii_serpentina_banda", "iii_serpentina_existentes",
                 "tabu_sin_existentes", "tabu_con_existentes"]
    etqs = ["i\nserpentina", "ii\nserp. banda", "iii\nserp. exist.", "tabú\nsin exist.", "tabú\ncon exist."]
    fig, axs = plt.subplots(len(insts), 2, figsize=(13, 4.6 * len(insts)), squeeze=False)
    niveles = [f"Z_nivel{k}" for k in range(1, 6)] + ["Z_otro"]
    col_niv = ["#1b6ca8", "#d95f02", "#1b9e77", "#7570b3", "#e6ab02", "#999999"]
    x = np.arange(len(variantes))
    for fila, inst in enumerate(insts):
        s = d[d["instancia"] == inst].set_index("variante").reindex(variantes)
        ax = axs[fila, 0]
        ax.bar(x, s["Z"], color=["#999999"] * 3 + ["#1b6ca8", "#1b9e77"])
        for xi, z, p in zip(x, s["Z"], s["pares_inmediatos_misma_especie"]):
            ax.annotate(f"{z:.0f}\n{int(p)} pares", (xi, z), ha="center", va="bottom", fontsize=8)
        ax.set_xticks(x)
        ax.set_xticklabels(etqs, fontsize=8)
        ax.set_ylabel("Z (competencia total)")
        ax.set_ylim(0, s["Z"].max() * 1.18)
        ax.set_title(f"Z por variante ({inst}); pares inmediatos de la misma especie")
        ax = axs[fila, 1]
        base = np.zeros(len(variantes))
        for c, col in zip(niveles, col_niv):
            ax.bar(x, s[c], bottom=base, color=col,
                   label=c.replace("Z_nivel", "nivel ").replace("Z_otro", "otro"))
            base += s[c].to_numpy()
        ax.set_xticks(x)
        ax.set_xticklabels(etqs, fontsize=8)
        ax.set_ylabel("Z por nivel de competencia")
        ax.set_title(f"Z por nivel de competencia 1-5 ({inst})")
        ax.set_ylim(0, base.max() * 1.15)
        if fila == 0:
            ax.legend(fontsize=8, ncol=6, loc="upper center")
    _guardar(fig, salida / "practica_distribucion.png")


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
