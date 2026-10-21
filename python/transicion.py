"""Orden de plantación en serpentina y matrices de transición entre especies (réplica en Python
de src/transicion.rs). Reglas idénticas a las de Rust:
- orden: filas por `fila` creciente; en la fila, `col` creciente si `fila` es par y decreciente si
  es impar; sólo plantas nuevas (especie >= 0 y no existente);
- vecindad inmediata: pares a distancia <= d + tol, ambas plantadas (nuevas o existentes), contados
  en los dos sentidos.
Patrones: columnas pos, fila, col, x, y, especie (-1 = vacía), existente (opcional)."""
from pathlib import Path

import numpy as np
import pandas as pd

try:
    from comun import RAIZ, leer_config
except ImportError:
    from python.comun import RAIZ, leer_config

J = 10
INSTANCIAS = ("1ha", "map_27")


def orden_serpentina(patron):
    p = patron
    if "existente" in p.columns:
        p = p[p["existente"] == 0]
    p = p[p["especie"] >= 0].copy()
    p["_c"] = np.where(p["fila"] % 2 == 0, p["col"], -p["col"])
    p = p.sort_values(["fila", "_c"], kind="stable")
    o = p[["pos", "fila", "col", "x", "y", "especie"]].reset_index(drop=True)
    o.insert(0, "paso", np.arange(1, len(o) + 1))
    return o


def _largo(conteos):
    n = conteos.sum(axis=1)
    f = [(j, k, int(conteos[j, k]), int(n[j]),
          float(conteos[j, k] / n[j]) if n[j] > 0 else 0.0)
         for j in range(J) for k in range(J)]
    return pd.DataFrame(f, columns=["de", "a", "conteo", "n", "prob"])


def matriz_orden(orden):
    e = orden["especie"].to_numpy(int)
    c = np.zeros((J, J), dtype=np.int64)
    np.add.at(c, (e[:-1], e[1:]), 1)
    return _largo(c)


def matriz_vecindad(patron, d, tol=1e-6):
    p = patron[patron["especie"] >= 0]
    e = p["especie"].to_numpy(int)
    xy = p[["x", "y"]].to_numpy(float)
    c = np.zeros((J, J), dtype=np.int64)
    for i in range(len(xy)):
        dist2 = ((xy[i + 1:] - xy[i]) ** 2).sum(axis=1)
        v = np.nonzero(dist2 <= (d + tol) ** 2)[0] + i + 1
        np.add.at(c, (np.full(len(v), e[i]), e[v]), 1)
        np.add.at(c, (e[v], np.full(len(v), e[i])), 1)
    return _largo(c)


def estacionaria(matriz):
    """Vector propio izquierdo de valor propio 1 de la matriz restringida a las especies con
    n > 0 (cada fila renormalizada sobre esas especies); las ausentes quedan en 0."""
    c = matriz.pivot(index="de", columns="a", values="conteo").reindex(
        index=range(J), columns=range(J), fill_value=0).to_numpy(float)
    pres = np.nonzero(c.sum(axis=1) > 0)[0]
    pi = pd.Series(0.0, index=range(J))
    if len(pres) == 0:
        return pi
    s = c[np.ix_(pres, pres)]
    suma = s.sum(axis=1)
    for i in np.nonzero(suma == 0)[0]:
        s[i, i] = 1.0  # fila cuyas transiciones sólo van a especies ausentes: absorbente
    P = s / s.sum(axis=1, keepdims=True)
    w, v = np.linalg.eig(P.T)
    vec = np.real(v[:, np.argmin(np.abs(w - 1))])
    vec = np.abs(vec) / np.abs(vec).sum()
    pi.iloc[pres] = vec
    return pi


def _composicion_patron(patron):
    e = patron["especie"][patron["especie"] >= 0]
    return pd.Series(e.value_counts().reindex(range(J), fill_value=0) / max(len(e), 1)).astype(float)


def main(origen=None, instancias=INSTANCIAS, salida=None):
    origen = Path(origen or RAIZ / "results")
    salida = Path(salida or origen)
    salida.mkdir(parents=True, exist_ok=True)
    d = float(leer_config()["d"])
    filas, est = [], []

    def agregar(inst, patron, alcance, largo_v, largo_o, comp):
        for tipo, m in (("vecindad", largo_v), ("orden", largo_o)):
            if m is None:
                continue
            t = m.copy()
            t.insert(0, "tipo", tipo)
            t.insert(0, "alcance", alcance)
            t.insert(0, "patron", patron)
            t.insert(0, "instancia", inst)
            filas.append(t)
        pi = estacionaria(largo_v)
        for j in range(J):
            est.append([inst, patron, alcance, j, float(pi[j]), float(comp[j])])

    for inst in instancias:
        ruta_t = origen / f"paso3_transicion_{inst}.csv"
        ruta_p = origen / f"paso3_plantas_{inst}.csv"
        if ruta_t.exists() and ruta_p.exists():
            t = pd.read_csv(ruta_t)
            comp = pd.read_csv(ruta_p).set_index("especie")["fraccion"].reindex(range(J), fill_value=0.0)
            v, o = (t[t["tipo"] == k].drop(columns="tipo").reset_index(drop=True)
                    for k in ("vecindad", "orden"))
            agregar(inst, "tabu_paso3", "todos_los_escenarios", v, o, comp)
        for patron, ruta in (("tabu_paso3", origen / f"paso3_patron_{inst}_omega0.csv"),
                             ("practica_iii", origen / f"practica_patron_{inst}_iii.csv")):
            if not ruta.exists():
                continue
            p = pd.read_csv(ruta)
            o = orden_serpentina(p)
            if patron == "tabu_paso3":
                o.to_csv(salida / f"orden_plantacion_{inst}.csv", index=False)
            agregar(inst, patron, "omega0", matriz_vecindad(p, d), matriz_orden(o),
                    _composicion_patron(p))
    if filas:
        pd.concat(filas, ignore_index=True).to_csv(salida / "transicion.csv", index=False)
        pd.DataFrame(est, columns=["instancia", "patron", "alcance", "especie", "pi_estacionaria",
                                   "composicion_plantada"]).to_csv(
            salida / "transicion_estacionaria.csv", index=False)


if __name__ == "__main__":
    main()
