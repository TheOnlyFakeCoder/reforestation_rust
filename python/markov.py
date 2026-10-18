"""Extensión Markov (spec 11): vivas/muertas a 1 y 2 años y reposición R_j."""
from pathlib import Path

import numpy as np
import pandas as pd

try:
    from comun import RAIZ, leer_especies
except ImportError:
    from python.comun import RAIZ, leer_especies

J = 10


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
    filas = []
    for inst in instancias:
        f = comp.loc[inst]
        for comp_nombre, pref in (("D*", "D_"), ("Dnom", "Dnom_")):
            n = f[[f"{pref}{j}" for j in range(J)]].to_numpy(float)
            v1, v2 = vivas_esperadas(n, s1, s2)
            for j in range(J):
                filas.append({"instancia": inst, "composicion": comp_nombre, "especie": esp[j],
                              "idx": j, "plantas": n[j], "muerte_ano1": m1[j],
                              "muerte_ano2": m2[j], "vivas_1ano": v1[j], "vivas_2anos": v2[j],
                              "R_j": R[j], "reposicion_esperada": n[j] * R[j]})
    df = pd.DataFrame(filas)
    df.to_csv(resultados / "markov.csv", index=False, float_format="%.6f")
    return df


if __name__ == "__main__":
    df = main()
    print(df[df.instancia == "1ha"].groupby("composicion")[
        ["plantas", "vivas_1ano", "vivas_2anos", "reposicion_esperada"]].sum())
