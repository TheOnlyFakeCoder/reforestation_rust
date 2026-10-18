"""Paso 2: compra de plantas por PLE con escenarios y regla del voceador."""
import math
import sys
import time
from pathlib import Path

import numpy as np
import pandas as pd
import pulp

try:
    from comun import RAIZ, leer_config, leer_especies
except ImportError:  # ejecución como paquete
    from python.comun import RAIZ, leer_config, leer_especies

J = 10
EPSILONES = (0.10, 0.0)
PI_FACTORES = (2, 4, 10)


def cargar_instancia(ruta):
    df = pd.read_csv(ruta)
    return df[[f"X_{j}" for j in range(J)]].to_numpy(dtype=float)


def escenarios_C(D, X):
    return np.asarray(D, dtype=float)[None, :] - np.asarray(X, dtype=float)


def voceador(C, c, pi_factor, eta=0.0):
    """Cuantil empírico de nivel (pi-c)/(pi+eta), entero y >= 0."""
    n = C.shape[0]
    q = np.zeros(J, dtype=int)
    for j in range(J):
        pi = pi_factor * c[j]
        nivel = (pi - c[j]) / (pi + eta)
        k = min(max(math.ceil(n * nivel - 1e-12), 1), n) - 1
        q[j] = max(0, math.ceil(np.sort(C[:, j])[k] - 1e-9))
    return q


def _solver():
    if hasattr(pulp, "HiGHS") and pulp.HiGHS().available():
        return pulp.HiGHS(msg=False)
    return pulp.PULP_CBC_CMD(msg=False)


def resolver_ple(C, c, pi_factor, presupuesto=None, eta=0.0):
    """PLE exacto; agrupa escenarios con el mismo C_j (equivale a la forma completa)."""
    n = C.shape[0]
    prob = pulp.LpProblem("paso2", pulp.LpMinimize)
    q = [prob.add_variable(f"q{j}", lowBound=0, cat="Integer") for j in range(J)]
    pen = []
    for j in range(J):
        pi = pi_factor * c[j]
        valores, cuentas = np.unique(C[:, j], return_counts=True)
        for k, (v, m) in enumerate(zip(valores, cuentas)):
            f = prob.add_variable(f"f{j}_{k}", lowBound=0)
            e = prob.add_variable(f"e{j}_{k}", lowBound=0)
            prob += f >= float(v) - q[j]
            prob += e >= q[j] - float(v)
            pen.append((float(m) * pi) * f + (float(m) * eta) * e)
    prob += pulp.lpSum(float(c[j]) * q[j] for j in range(J)) + pulp.lpSum(pen) * (1.0 / n)
    if presupuesto is not None:
        prob += pulp.lpSum(float(c[j]) * q[j] for j in range(J)) <= presupuesto
    prob.solve(_solver())
    qv = np.array([int(round(v.value())) for v in q])
    return qv, float(pulp.value(prob.objective))


def metricas(C, c, q, pi_factor, eta=0.0):
    c = np.asarray(c, dtype=float)
    q = np.asarray(q)
    falt = np.maximum(C - q[None, :], 0)
    sobr = np.maximum(q[None, :] - C, 0)
    costo = float(c @ q)
    pi = pi_factor * c
    obj = costo + float((falt * pi[None, :]).sum(axis=1).mean()
                        + eta * sobr.sum(axis=1).mean())
    return {"costo": costo, "faltante": float(falt.sum(axis=1).mean()),
            "sobrante": float(sobr.sum(axis=1).mean()), "objetivo": obj}


def correr(resultados=None, salida=None):
    res = Path(resultados or RAIZ / "results")
    cfg = leer_config()
    c = leer_especies()["c"].to_numpy(dtype=float)
    comp = pd.read_csv(res / "paso1_composicion.csv")
    filas = []
    for _, r in comp.iterrows():
        ruta = res / f"escenarios_X_{r['instancia']}.csv"
        if not ruta.exists():
            continue
        X = cargar_instancia(ruta)
        B = cfg["presupuesto_ha"] * float(r["area_ha"])
        for eps in EPSILONES:
            pref = "D_" if eps > 0 else "Dnom_"
            D = np.array([r[f"{pref}{j}"] for j in range(J)], dtype=float)
            C = escenarios_C(D, X)
            for pf in PI_FACTORES:
                q, _ = resolver_ple(C, c, pf, presupuesto=B)
                m = metricas(C, c, q, pf)
                qv = voceador(C, c, pf)
                fila = {"instancia": r["instancia"], "map_id": r["map_id"],
                        "epsilon": eps, "pi_factor": pf}
                fila.update({f"q_{j}": int(q[j]) for j in range(J)})
                fila.update({"costo": m["costo"], "costo_ha": m["costo"] / r["area_ha"],
                             "presupuesto": B, "cabe": bool(m["costo"] <= B + 1e-6),
                             "faltante_esperado": m["faltante"],
                             "sobrante_esperado": m["sobrante"], "objetivo": m["objetivo"]})
                fila.update({f"q_voceador_{j}": int(qv[j]) for j in range(J)})
                fila["costo_voceador"] = float(c @ qv)
                filas.append(fila)
    out = Path(salida or res / "paso2_compra.csv")
    pd.DataFrame(filas).to_csv(out, index=False, float_format="%.6f")
    return len(filas)


if __name__ == "__main__":
    t0 = time.time()
    n = correr()
    print(f"{n} filas en {time.time() - t0:.1f} s")
