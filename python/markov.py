"""Extensión Markov (spec 11): vivas/muertas a 1 y 2 años y reposición R_j."""
from pathlib import Path

import numpy as np
import pandas as pd

try:
    from comun import RAIZ, leer_config, leer_especies
except ImportError:
    from python.comun import RAIZ, leer_config, leer_especies

J = 10
INSTANCIAS_COMPRA = ("1ha", "map_27")


def muertes(s1, s2):
    """Probabilidad de muerte en el año 1 y en el año 2 (dado vivo al año 1)."""
    s1, s2 = np.asarray(s1, float), np.asarray(s2, float)
    return 1 - s1, 1 - s2 / s1


def vivas_esperadas(n, s1, s2):
    n = np.asarray(n, float)
    return n * np.asarray(s1, float), n * np.asarray(s2, float)


def reposicion(s1, s2):
    s1, s2 = np.asarray(s1, float), np.asarray(s2, float)
    return (1 - s1) + (s1 - s2) + (1 - s1) ** 2


def plantas_compradas(q, D, X):
    """n_j esperado = media sobre escenarios de min(q_j, C+_j) con C = D - X."""
    Cpos = np.maximum(np.asarray(D, float)[None, :] - np.asarray(X, float), 0.0)
    return np.minimum(np.asarray(q, float)[None, :], Cpos).mean(axis=0)


def main(resultados=None, instancias=None, origen=None):
    origen = Path(origen or RAIZ / "results")
    resultados = Path(resultados or RAIZ / "results")
    esp = leer_especies()["especie"].tolist()
    sup = pd.read_csv(RAIZ / "data" / "supervivencia.csv")
    s1, s2 = sup["s1"].to_numpy(float), sup["s2"].to_numpy(float)
    m1, m2 = muertes(s1, s2)
    R = reposicion(s1, s2)
    comp = pd.read_csv(origen / "paso1_composicion.csv").set_index("instancia")
    if instancias is None:
        instancias = list(comp.index)
    cfg = leer_config()
    p2 = pd.read_csv(origen / "paso2_compra.csv")
    filas = []

    def agregar(inst, comp_nombre, base, n):
        v1, v2 = vivas_esperadas(n, s1, s2)
        for j in range(J):
            filas.append({"instancia": inst, "composicion": comp_nombre, "base": base,
                          "especie": esp[j], "idx": j, "plantas": n[j], "muerte_ano1": m1[j],
                          "muerte_ano2": m2[j], "vivas_1ano": v1[j], "vivas_2anos": v2[j],
                          "R_j": R[j], "reposicion_esperada": n[j] * R[j]})

    for inst in instancias:
        f = comp.loc[inst]
        # D_completo: si se plantara todo D (sin plantas existentes); sobrestima lo plantado.
        for comp_nombre, pref in (("D*", "D_"), ("Dnom", "Dnom_")):
            agregar(inst, comp_nombre, "D_completo", f[[f"{pref}{j}" for j in range(J)]].to_numpy(float))
        # Plantas realmente plantadas: min(q_j, C+_j) promedio sobre los escenarios.
        ruta_x = origen / f"escenarios_X_{inst}.csv"
        if inst in INSTANCIAS_COMPRA and ruta_x.exists():
            X = pd.read_csv(ruta_x)[[f"X_{j}" for j in range(J)]].to_numpy(float)
            Dstar = f[[f"D_{j}" for j in range(J)]].to_numpy(float)
            Dnom = f[[f"Dnom_{j}" for j in range(J)]].to_numpy(float)
            r = p2[(p2["instancia"] == inst) & np.isclose(p2["epsilon"], cfg["epsilon"])
                   & np.isclose(p2["pi_factor"], cfg["pi_factor"])]
            if len(r):
                q = r.iloc[0][[f"q_{j}" for j in range(J)]].to_numpy(float)
                agregar(inst, "D*", "compra_d", plantas_compradas(q, Dstar, X))
            agregar(inst, "Dnom", "compra_nominal", plantas_compradas(Dnom, Dnom, X))
    df = pd.DataFrame(filas)
    df.to_csv(resultados / "markov.csv", index=False, float_format="%.6f")
    return df


if __name__ == "__main__":
    df = main()
    print(df[df.instancia == "1ha"].groupby(["base", "composicion"])[
        ["plantas", "vivas_1ano", "vivas_2anos", "reposicion_esperada"]].sum())
