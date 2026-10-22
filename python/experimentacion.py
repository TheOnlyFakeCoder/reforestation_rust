"""Experimentación (1000 simulaciones): valores esperados por hectárea con IC 95 %.

Por escenario de Monte Carlo (`escenarios_X_<inst>.csv`): T (existentes), X_j, C_j = max(D*_j − X_j, 0)
y n_j = min(q_j, C_j), con q de `paso2_compra.csv` (ε = 0.1, π_factor del config); q_j es constante.
Del paso 3 (`paso3_escenarios.csv`): Z, Z_nivel1..5 y Z_otro. Todo se divide entre `area_ha`.
IC 95 %: media ± 1.96·sd/√n (sd muestral, n escenarios)."""
import argparse
from pathlib import Path

import numpy as np
import pandas as pd

try:
    from comun import RAIZ, leer_config
except ImportError:  # ejecución como paquete
    from python.comun import RAIZ, leer_config

J = 10
INSTANCIAS = ("1ha", "map_27")
EPSILON = 0.1
COLUMNAS = ["instancia", "magnitud", "especie", "n", "media_ha", "sd_ha", "ic95_inf", "ic95_sup"]
NIVELES = [f"Z_nivel{k}" for k in range(1, 6)] + ["Z_otro"]


def _fila(inst, magnitud, especie, valores, area):
    v = np.asarray(valores, dtype=float) / area
    n = len(v)
    media = float(v.mean()) if n else np.nan
    sd = float(v.std(ddof=1)) if n > 1 else (0.0 if n == 1 else np.nan)
    h = 1.96 * sd / np.sqrt(n) if n else np.nan
    return dict(instancia=inst, magnitud=magnitud, especie=especie, n=n, media_ha=media,
                sd_ha=sd, ic95_inf=media - h, ic95_sup=media + h)


def tabla(origen, instancias=INSTANCIAS):
    origen = Path(origen)
    pi = float(leer_config()["pi_factor"])
    comp = pd.read_csv(origen / "paso1_composicion.csv").set_index("instancia")
    p2 = pd.read_csv(origen / "paso2_compra.csv")
    p2 = p2[np.isclose(p2["epsilon"], EPSILON) & np.isclose(p2["pi_factor"], pi)]
    p2 = p2.set_index("instancia")
    p3 = pd.read_csv(origen / "paso3_escenarios.csv")
    filas = []
    for inst in instancias:
        area = float(comp.loc[inst, "area_ha"])
        D = comp.loc[inst, [f"D_{j}" for j in range(J)]].to_numpy(float)
        q = p2.loc[inst, [f"q_{j}" for j in range(J)]].to_numpy(float)
        esc = pd.read_csv(origen / f"escenarios_X_{inst}.csv")
        X = esc[[f"X_{j}" for j in range(J)]].to_numpy(float)
        C = np.maximum(D - X, 0)
        N = np.minimum(q, C)
        filas.append(_fila(inst, "T", None, esc["T"], area))
        for nombre, m in (("X", X), ("C", C), ("n", N)):
            filas += [_fila(inst, nombre, j, m[:, j], area) for j in range(J)]
        filas += [_fila(inst, "q", j, np.full(len(X), q[j]), area) for j in range(J)]
        z = p3[p3["instancia"] == inst]
        filas += [_fila(inst, c, None, z[c], area) for c in ["Z"] + NIVELES if c in z.columns]
    t = pd.DataFrame(filas, columns=COLUMNAS)
    t["especie"] = t["especie"].astype("Int64")
    return t


def main(origen=None, destino=None, instancias=INSTANCIAS):
    origen = Path(origen or RAIZ / "results")
    destino = Path(destino or origen)
    destino.mkdir(parents=True, exist_ok=True)
    ruta = destino / "experimentacion.csv"
    tabla(origen, instancias).to_csv(ruta, index=False)
    return ruta


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--origen")
    ap.add_argument("--destino")
    a = ap.parse_args()
    print(main(a.origen, a.destino))
