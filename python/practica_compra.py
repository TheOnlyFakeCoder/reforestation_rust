"""Práctica actual: compra nominal (a, b) vs. paso 2 (c, d), evaluada en los escenarios."""
from pathlib import Path

import numpy as np
import pandas as pd

try:
    from comun import RAIZ, leer_config, leer_especies
except ImportError:
    from python.comun import RAIZ, leer_config, leer_especies

J = 10
NOMBRES = {"a": "nominal 658/ha", "b": "nominal 524/ha",
           "c": "paso 2 (eps=0)", "d": "paso 2 (eps=0.1)"}


def parametros():
    """Presupuesto por hectárea y factor π leídos de config/default.json."""
    cfg = leer_config()
    return {"presupuesto_ha": float(cfg["presupuesto_ha"]), "pi_factor": float(cfg["pi_factor"]),
            "epsilon": float(cfg["epsilon"])}


def mayor_residuo(total, pesos, suma_pesos):
    """Reparte `total` enteros en proporción pesos/suma_pesos (aritmética entera,
    empates al índice menor)."""
    pesos = [int(w) for w in pesos]
    base = [total * w // suma_pesos for w in pesos]
    resto = [total * w % suma_pesos for w in pesos]
    faltan = total - sum(base)
    orden = sorted(range(len(pesos)), key=lambda j: (-resto[j], j))
    for j in orden[:faltan]:
        base[j] += 1
    return np.array(base, dtype=int)


def evaluar(q, D, X, c, s1, presupuesto, area):
    """Métricas de compra q contra escenarios X con requerimiento D (C = D - X)."""
    q = np.asarray(q, dtype=float)
    C = np.asarray(D, dtype=float)[None, :] - np.asarray(X, dtype=float)
    Cpos = np.maximum(C, 0.0)
    p = np.minimum(q[None, :], Cpos)
    costo = float(np.dot(c, q))
    vivas = (p * s1[None, :]).sum(axis=1)
    tot = p.sum(axis=1)
    sup = np.where(tot > 0, vivas / np.where(tot > 0, tot, 1), 0.0)
    return {"costo_ha": costo / area, "cabe": bool(costo <= presupuesto + 1e-9),
            "faltante_medio": float((Cpos - p).sum(axis=1).mean()),
            "sobrante_medio": float((q[None, :] - p).sum(axis=1).mean()),
            "supervivencia": float(sup.mean()), "vivas_1ano": float(vivas.mean())}


def main(resultados=None, instancias=None, origen=None):
    origen = Path(origen or RAIZ / "results")
    resultados = Path(resultados or RAIZ / "results")
    resultados.mkdir(parents=True, exist_ok=True)
    esp = leer_especies()
    c = esp["c"].to_numpy(dtype=float)
    delta = esp["delta"].to_numpy()
    s1 = pd.read_csv(RAIZ / "data" / "supervivencia.csv")["s1"].to_numpy(dtype=float)
    comp = pd.read_csv(origen / "paso1_composicion.csv").set_index("instancia")
    p2 = pd.read_csv(origen / "paso2_compra.csv")
    par = parametros()
    p2 = p2[np.isclose(p2["pi_factor"], par["pi_factor"])]
    if instancias is None:
        instancias = [i for i in comp.index if (origen / f"escenarios_X_{i}.csv").exists()]
    filas = []
    for inst in instancias:
        f = comp.loc[inst]
        area = float(f["area_ha"])
        Dstar = f[[f"D_{j}" for j in range(J)]].to_numpy(dtype=float)
        Dnom = f[[f"Dnom_{j}" for j in range(J)]].to_numpy(dtype=float)
        X = pd.read_csv(origen / f"escenarios_X_{inst}.csv")[[f"X_{j}" for j in range(J)]]
        X = X.to_numpy(dtype=float)
        B = par["presupuesto_ha"] * area
        opciones = {"a": (Dnom, Dnom),
                    "b": (mayor_residuo(round(524 * area), delta, 658), Dnom)}
        for op, eps in (("c", 0.0), ("d", par["epsilon"])):
            r = p2[(p2["instancia"] == inst) & (np.isclose(p2["epsilon"], eps))]
            if len(r):
                q = r.iloc[0][[f"q_{j}" for j in range(J)]].to_numpy(dtype=float)
                opciones[op] = (q, Dnom if op == "c" else Dstar)
        for op, (q, D) in opciones.items():
            m = evaluar(q, D, X, c, s1, B, area)
            filas.append({"instancia": inst, "map_id": int(f["map_id"]), "area_ha": area,
                          "opcion": op, "descripcion": NOMBRES[op],
                          "plantas_compradas": int(round(float(np.sum(q)))),
                          "escenarios": len(X), **m})
    df = pd.DataFrame(filas)
    df.to_csv(resultados / "practica_compra.csv", index=False, float_format="%.6f")
    return df


if __name__ == "__main__":
    print(main().query("instancia in ['1ha','map_27']").to_string())
