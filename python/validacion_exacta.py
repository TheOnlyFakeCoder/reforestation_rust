"""Validación exacta con CP-SAT de los modelos espaciales (paso 1, ec. 21; paso 3, ec. 24).

Lee las instancias pequeñas de `results/validacion/inst_<id>.json` y las corridas del tabú
`tabu_<id>.csv` (las escriben `reto instancias-val` y `reto tabu-val`), resuelve cada una con
CP-SAT y escribe `results/validacion_exacta.csv`.

- Z = Σ sobre aristas (u, v) de peso[anillo][φu][φv] cuando ambas posiciones están plantadas
  (pares libre–libre, libre–existente y existente–existente), igual que `z_total` en Rust.
- Objetivo: paso 1 γ·Z + Σ θ_j·D_j; paso 3 Z (γ = 1, θ = 0).
- El modelo CP-SAT usa coeficientes enteros round(γ·w·h·escala) y round(θ·escala) (w·h es
  `peso`); Z*, Z_TS y las brechas se calculan con los coeficientes originales.
- Antes de comparar se verifica la consistencia Rust↔Python (1e-6): Z de Python de cada solución
  tabú = z de Rust, y la solución de CP-SAT evaluada con `reto evaluar-val` = Z de Python. Si
  falla, se escribe la discrepancia y el programa sale con código 2.
"""
import argparse
import json
import math
import os
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
import pandas as pd
from ortools.sat.python import cp_model

try:
    from comun import RAIZ
except ImportError:
    from python.comun import RAIZ

ESCALA = 10_000
TOL_CONSISTENCIA = 1e-6
N_ESPECIES = 10
RETO = RAIZ / "target" / "release" / "reto.exe"
WORKERS = os.cpu_count() or 8  # todos los hilos lógicos (CP-SAT reparte su portafolio entre ellos)
COLUMNAS = [
    "id", "modelo", "formulacion", "hint", "n_pos", "existentes", "n_especies_libres", "aristas",
    "variables", "z_opt", "cota", "optimo_probado", "tiempo_cpsat_s", "z_ts_mejor", "z_ts_media",
    "z_ts_sd", "tiempo_ts_medio_s", "brecha_mejor", "brecha_media", "frac_semillas_optimo",
    "z_comp_opt", "z_comp_ts_mejor", "brecha_z", "error_redondeo_max",
    # añadidas al final: estado de CP-SAT, cota del error total de redondeo de una solución
    # (Σ por término del mayor error) y brecha de la mejor semilla contra la incumbente.
    "estado", "tolerancia_redondeo", "brecha_vs_incumbente", "tiempo_limite_s", "workers",
]


# ---------------------------------------------------------------- lectura y evaluación

def leer_instancia(ruta):
    with open(ruta, encoding="utf-8") as f:
        return json.load(f)


def leer_tabu(ruta):
    return pd.read_csv(ruta, dtype={"phi": str})


def leer_phi(texto):
    return [int(s) for s in str(texto).split(";")]


def texto_phi(phi):
    return ";".join(str(int(c)) for c in phi)


def z_python(inst, phi):
    """Z con los coeficientes originales, en el mismo orden de aristas que `z_total`."""
    peso = inst["peso"]
    z = 0.0
    for u, v, a in inst["aristas"]:
        p, q = phi[u], phi[v]
        if p >= 0 and q >= 0:
            z += peso[a][p][q]
    return z


def objetivo_python(inst, phi):
    mortalidad = sum(inst["theta"][c] for c in phi if c >= 0)
    return inst["gamma"] * z_python(inst, phi) + mortalidad


def violaciones(inst, phi):
    """Restricciones que φ no cumple (lista vacía = factible); mismas reglas que Rust."""
    if len(phi) != inst["n_pos"]:
        return [f"φ tiene {len(phi)} posiciones y la malla {inst['n_pos']}"]
    v = []
    cuenta = [0] * N_ESPECIES
    for u, c in enumerate(phi):
        if not -1 <= c < N_ESPECIES:
            return [f"posición {u}: especie {c} fuera de -1..9"]
        f = inst["fija"][u]
        if f >= 0:
            if c != f:
                v.append(f"posición {u}: la existente {f} cambió a {c}")
        elif c >= 0:
            cuenta[c] += 1
    total = sum(cuenta)
    if inst["modelo"] == "paso3":
        for j in range(N_ESPECIES):
            if cuenta[j] > inst["cuotas"][j]:
                v.append(f"especie {j}: {cuenta[j]} nuevas > n = {inst['cuotas'][j]}")
        if total != inst["m"]:
            v.append(f"{total} nuevas ≠ M = {inst['m']}")
    else:
        for j in range(N_ESPECIES):
            if not inst["banda_l"][j] <= cuenta[j] <= inst["banda_u"][j]:
                v.append(f"especie {j}: D = {cuenta[j]} fuera de "
                         f"[{inst['banda_l'][j]}, {inst['banda_u'][j]}]")
        if total != inst["d_tot"]:
            v.append(f"{total} plantas ≠ D_tot = {inst['d_tot']}")
    return v


# ---------------------------------------------------------------- modelo CP-SAT

class _Modelo:
    """CpModel con lo necesario para recuperar la solución en unidades originales."""

    def __init__(self, inst, escala, formulacion, hint):
        if formulacion not in ("basica", "ajustada"):
            raise ValueError(f"formulación desconocida: {formulacion}")
        self.escala = escala
        self.formulacion = formulacion
        n = inst["n_pos"]
        fija, peso, g, theta = inst["fija"], inst["peso"], inst["gamma"], inst["theta"]
        paso3 = inst["modelo"] == "paso3"
        self.libres = [v for v in range(n) if fija[v] < 0]
        if paso3:
            activas = [j for j in range(N_ESPECIES) if inst["cuotas"][j] > 0]
        else:
            activas = [j for j in range(N_ESPECIES) if inst["banda_u"][j] > 0]
        m = cp_model.CpModel()
        x = {v: {j: m.NewBoolVar(f"x_{v}_{j}") for j in activas} for v in self.libres}

        # Restricciones de asignación.
        if paso3:
            todas_plantadas = False
            for v in self.libres:
                m.Add(sum(x[v].values()) <= 1)
            for j in activas:
                m.Add(sum(x[v][j] for v in self.libres) <= inst["cuotas"][j])
            m.Add(sum(x[v][j] for v in self.libres for j in activas) == inst["m"])
        else:
            todas_plantadas = inst["d_tot"] == len(self.libres)
            for v in self.libres:
                if todas_plantadas:
                    m.AddExactlyOne(x[v].values())
                else:
                    m.Add(sum(x[v].values()) <= 1)
            for j in activas:
                s = sum(x[v][j] for v in self.libres)
                m.Add(s >= inst["banda_l"][j])
                m.Add(s <= inst["banda_u"][j])
            m.Add(sum(x[v][j] for v in self.libres for j in activas) == inst["d_tot"])

        # Coeficientes del objetivo (originales): constante, lineales y libre–libre.
        constante = sum(theta[fija[v]] for v in range(n) if fija[v] >= 0)
        lineal = {v: {j: theta[j] for j in activas} for v in self.libres}
        libre_libre = []
        for u, w, a in inst["aristas"]:
            fu, fw = fija[u], fija[w]
            if fu >= 0 and fw >= 0:
                constante += g * peso[a][fu][fw]
            elif fu >= 0:
                for k in activas:
                    lineal[w][k] += g * peso[a][fu][k]
            elif fw >= 0:
                for j in activas:
                    lineal[u][j] += g * peso[a][j][fw]
            else:
                libre_libre.append((u, w, a))

        def entero(c):
            ci = round(c * escala)
            err = abs(ci / escala - c)
            return ci, err

        terminos = []
        errores = []
        tolerancia = 0.0
        y_de = []  # (u, w, j, k, var) para el hint
        for v in self.libres:
            peor = 0.0
            for j in activas:
                ci, err = entero(lineal[v][j])
                errores.append(err)
                peor = max(peor, err)
                if ci:
                    terminos.append(ci * x[v][j])
            tolerancia += peor
        for u, w, a in libre_libre:
            peor = 0.0
            y = {}
            for j in activas:
                for k in activas:
                    ci, err = entero(g * peso[a][j][k])
                    errores.append(err)
                    peor = max(peor, err)
                    if formulacion == "basica":
                        if ci > 0:
                            var = m.NewBoolVar(f"y_{u}_{w}_{j}_{k}")
                            m.Add(var >= x[u][j] + x[w][k] - 1)
                            terminos.append(ci * var)
                            y_de.append((u, w, j, k, var))
                    else:
                        var = m.NewBoolVar(f"y_{u}_{w}_{j}_{k}")
                        y[j, k] = var
                        if ci:
                            terminos.append(ci * var)
                        y_de.append((u, w, j, k, var))
            tolerancia += peor
            if formulacion == "ajustada":
                # Transporte entre las especies de u y de w. Paso 1 (todas plantadas): igualdades.
                # Paso 3: ≤ y, para que y = x_uj·x_wk cuando ambas están plantadas,
                # Σ y ≥ p_u + p_w − 1 (proyección de las igualdades con la vacía como especie).
                for j in activas:
                    fila = sum(y[j, k] for k in activas)
                    m.Add(fila == x[u][j]) if todas_plantadas else m.Add(fila <= x[u][j])
                for k in activas:
                    col = sum(y[j, k] for j in activas)
                    m.Add(col == x[w][k]) if todas_plantadas else m.Add(col <= x[w][k])
                if not todas_plantadas:
                    m.Add(sum(y.values()) >= sum(x[u].values()) + sum(x[w].values()) - 1)
        m.Minimize(sum(terminos))

        self.hint = hint is not None
        if hint is not None:
            for v in self.libres:
                for j, var in x[v].items():
                    m.AddHint(var, int(hint[v] == j))
            for u, w, j, k, var in y_de:
                m.AddHint(var, int(hint[u] == j and hint[w] == k))

        self.model, self.x = m, x
        self.fija = list(fija)
        self.constante = constante
        self.error_redondeo_max = max(errores) if errores else 0.0
        self.tolerancia_redondeo = tolerancia


def modelo_cpsat(inst, escala=ESCALA, formulacion="basica", hint=None):
    """(CpModel, x) con x[v][j] booleanas para cada posición libre v y especie activa j."""
    mod = _Modelo(inst, escala, formulacion, hint)
    return mod.model, mod.x


def resolver(inst, tiempo=600, workers=8, formulacion="basica", hint=None, escala=ESCALA):
    """Resuelve con CP-SAT. Z, objetivo y cota en unidades originales; la cota es la del modelo
    escalado (BestObjectiveBound/escala + constante)."""
    mod = _Modelo(inst, escala, formulacion, hint)
    solver = cp_model.CpSolver()
    solver.parameters.max_time_in_seconds = float(tiempo)
    solver.parameters.num_workers = int(workers)
    estado = solver.Solve(mod.model)
    nombre = solver.StatusName(estado)
    if estado in (cp_model.INFEASIBLE, cp_model.MODEL_INVALID):
        raise RuntimeError(f"{inst.get('id')}: CP-SAT {nombre}")
    phi = None
    if estado in (cp_model.OPTIMAL, cp_model.FEASIBLE):
        phi = list(mod.fija)
        for v, por_especie in mod.x.items():
            for j, var in por_especie.items():
                if solver.Value(var):
                    phi[v] = j
    return {
        "phi": phi,
        "z": z_python(inst, phi) if phi is not None else math.nan,
        "objetivo": objetivo_python(inst, phi) if phi is not None else math.nan,
        "cota": solver.BestObjectiveBound() / escala + mod.constante,
        "optimo_probado": estado == cp_model.OPTIMAL,
        "estado": nombre,
        "tiempo": solver.WallTime(),
        "n_variables": len(mod.model.Proto().variables),
        "error_redondeo_max": mod.error_redondeo_max,
        "tolerancia_redondeo": mod.tolerancia_redondeo,
        "formulacion": formulacion,
        "hint": mod.hint,
        "tiempo_limite": float(tiempo),
        "workers": int(workers),
    }


# ---------------------------------------------------------------- comparación

def brecha(z, ref):
    """(z − ref)/ref; infinita si ref ≤ 0 (cota trivial: brecha relativa no acotada)."""
    if math.isnan(ref) or math.isnan(z):
        return math.nan
    if ref <= 0:
        return 0.0 if z == ref else math.inf
    return (z - ref) / ref


def fila_resultado(inst, tabu, r):
    """Fila de `validacion_exacta.csv`. En paso 1 los z_* son el objetivo γZ + Σθ·D. Sin óptimo
    probado la brecha se mide contra la cota (y `optimo_probado` = False lo marca)."""
    phis = [leer_phi(t) for t in tabu["phi"]]
    obj = np.array([objetivo_python(inst, p) for p in phis])
    zc = np.array([z_python(inst, p) for p in phis])
    i = int(np.argmin(obj))
    z_opt = r["objetivo"]
    ref = z_opt if r["optimo_probado"] else r["cota"]
    brecha_mejor = brecha(float(obj.min()), ref)
    if inst["modelo"] == "paso1":
        g = inst["gamma"]
        brecha_z = brecha(g * float(zc[i]), g * r["z"])
    else:
        brecha_z = brecha_mejor
    tol = TOL_CONSISTENCIA * max(1.0, abs(z_opt)) if not math.isnan(z_opt) else -1.0
    return {
        "id": inst["id"], "modelo": inst["modelo"], "formulacion": r["formulacion"],
        "hint": bool(r["hint"]), "n_pos": inst["n_pos"],
        "existentes": sum(1 for f in inst["fija"] if f >= 0),
        "n_especies_libres": inst.get("n_especies_libres"), "aristas": len(inst["aristas"]),
        "variables": r["n_variables"], "z_opt": z_opt, "cota": r["cota"],
        "optimo_probado": bool(r["optimo_probado"]), "tiempo_cpsat_s": r["tiempo"],
        "z_ts_mejor": float(obj.min()), "z_ts_media": float(obj.mean()),
        "z_ts_sd": float(obj.std(ddof=1)) if len(obj) > 1 else math.nan,
        "tiempo_ts_medio_s": float(tabu["tiempo_s"].mean()),
        "brecha_mejor": brecha_mejor, "brecha_media": brecha(float(obj.mean()), ref),
        "frac_semillas_optimo": float(np.mean(np.abs(obj - z_opt) <= tol)),
        "z_comp_opt": r["z"], "z_comp_ts_mejor": float(zc[i]), "brecha_z": brecha_z,
        "error_redondeo_max": r["error_redondeo_max"], "estado": r["estado"],
        "tolerancia_redondeo": r["tolerancia_redondeo"],
        "brecha_vs_incumbente": brecha(float(obj.min()), z_opt),
        "tiempo_limite_s": r["tiempo_limite"], "workers": r["workers"],
    }


def escribir_csv(filas, salida):
    salida = Path(salida)
    salida.parent.mkdir(parents=True, exist_ok=True)
    pd.DataFrame(filas, columns=COLUMNAS).to_csv(salida, index=False, float_format="%.12g")


# ---------------------------------------------------------------- consistencia Rust↔Python

def verificar_tabu(inst, tabu):
    """Discrepancias entre el z/objetivo de Rust (CSV) y los de Python, y soluciones infactibles."""
    d = []
    for _, t in tabu.iterrows():
        phi = leer_phi(t["phi"])
        z, o = z_python(inst, phi), objetivo_python(inst, phi)
        if abs(z - t["z"]) > TOL_CONSISTENCIA:
            d.append(f"{inst['id']} semilla {t['semilla']}: z Rust {t['z']!r} vs Python {z!r}")
        if abs(o - t["objetivo"]) > TOL_CONSISTENCIA:
            d.append(f"{inst['id']} semilla {t['semilla']}: objetivo Rust {t['objetivo']!r} "
                     f"vs Python {o!r}")
        for falla in violaciones(inst, phi):
            d.append(f"{inst['id']} semilla {t['semilla']}: solución tabú infactible: {falla}")
    return d


def evaluar_rust(reto, ruta_inst, phis, tmp):
    """[(z, objetivo, avisos)] de `reto evaluar-val` para cada asignación."""
    tmp = Path(tmp)
    tmp.mkdir(parents=True, exist_ok=True)
    ruta_phi = tmp / f"phi_{Path(ruta_inst).stem}.csv"
    pd.DataFrame({"phi": [texto_phi(p) for p in phis]}).to_csv(ruta_phi, index=False)
    r = subprocess.run([str(reto), "evaluar-val", "--inst", str(ruta_inst), "--phi", str(ruta_phi)],
                       capture_output=True, text=True, encoding="utf-8", cwd=RAIZ)
    if r.returncode != 0:
        raise RuntimeError(f"reto evaluar-val falló ({r.returncode}): {r.stderr}")
    lineas = r.stdout.strip().splitlines()
    if lineas[0].strip() != "z,objetivo" or len(lineas) != len(phis) + 1:
        raise RuntimeError(f"salida inesperada de reto evaluar-val: {r.stdout!r}")
    avisos = r.stderr.strip()
    return [(float(z), float(o), avisos) for z, o in (l.split(",") for l in lineas[1:])]


def verificar_cpsat(inst, ruta_inst, r, reto, tmp):
    phi = r["phi"]
    d = [f"{inst['id']} CP-SAT ({r['formulacion']}): infactible: {f}" for f in violaciones(inst, phi)]
    (z_r, o_r, avisos), = evaluar_rust(reto, ruta_inst, [phi], tmp)
    if avisos:
        d.append(f"{inst['id']} CP-SAT ({r['formulacion']}): avisos de Rust: {avisos}")
    if abs(z_r - r["z"]) > TOL_CONSISTENCIA:
        d.append(f"{inst['id']} CP-SAT ({r['formulacion']}): z Rust {z_r!r} vs Python {r['z']!r}")
    if abs(o_r - r["objetivo"]) > TOL_CONSISTENCIA:
        d.append(f"{inst['id']} CP-SAT ({r['formulacion']}): objetivo Rust {o_r!r} "
                 f"vs Python {r['objetivo']!r}")
    return d


def _discrepancias(salida, d):
    ruta = Path(salida).with_name(Path(salida).stem + "_discrepancias.txt")
    ruta.write_text("\n".join(d) + "\n", encoding="utf-8")
    print(f"DISCREPANCIA Rust↔Python ({len(d)}), ver {ruta}:", file=sys.stderr)
    for linea in d:
        print("  " + linea, file=sys.stderr)
    return 2


# ---------------------------------------------------------------- flujo completo

def validar(dir_val, salida, tiempo=600, workers=8, reto=RETO, ids=None,
            formulaciones=("basica", "ajustada"), actualizar=False):
    """Corre CP-SAT en cada instancia con las `formulaciones` en orden (la siguiente sólo si la
    anterior no cierra) y escribe `salida` tras cada fila. Con `actualizar`, conserva las filas
    de `salida` cuyo (id, formulación) no se vuelve a correr. Devuelve 0, o 2 si falla la
    consistencia Rust↔Python."""
    dir_val = Path(dir_val)
    rutas = sorted(dir_val.glob("inst_*.json"))
    if ids:
        rutas = [p for p in rutas if p.stem[len("inst_"):] in set(ids)]
    if not rutas:
        raise FileNotFoundError(f"no hay instancias inst_*.json en {dir_val}")
    datos = []
    d = []
    for ruta in rutas:
        inst = leer_instancia(ruta)
        tabu = leer_tabu(dir_val / f"tabu_{inst['id']}.csv")
        d += verificar_tabu(inst, tabu)
        datos.append((ruta, inst, tabu))
    if d:
        return _discrepancias(salida, d)
    print(f"consistencia tabú Rust↔Python: {sum(len(t) for _, _, t in datos)} soluciones OK",
          flush=True)
    filas = []
    previas = []
    if actualizar and Path(salida).exists():
        previas = pd.read_csv(salida).to_dict("records")
    orden = {"basica": 0, "ajustada": 1}

    def escribir():
        nuevas = {(f["id"], f["formulacion"]) for f in filas}
        todas = [f for f in previas if (f["id"], f["formulacion"]) not in nuevas] + filas
        todas.sort(key=lambda f: (str(f["id"]), orden.get(f["formulacion"], 2)))
        escribir_csv(todas, salida)

    with tempfile.TemporaryDirectory() as tmp:
        for ruta, inst, tabu in datos:
            hint = leer_phi(tabu.loc[tabu["objetivo"].idxmin(), "phi"])
            for formulacion in formulaciones:
                r = resolver(inst, tiempo=tiempo, workers=workers, formulacion=formulacion, hint=hint)
                if r["phi"] is not None:
                    d = verificar_cpsat(inst, ruta, r, reto, tmp)
                    if d:
                        escribir()
                        return _discrepancias(salida, d)
                filas.append(fila_resultado(inst, tabu, r))
                escribir()
                f = filas[-1]
                print(f"{inst['id']} {formulacion}: {r['estado']} z_opt={f['z_opt']:.9g} "
                      f"cota={f['cota']:.9g} t={r['tiempo']:.1f}s z_ts_mejor={f['z_ts_mejor']:.9g} "
                      f"brecha_mejor={f['brecha_mejor']:.3e}", flush=True)
                if r["optimo_probado"]:
                    break
    return 0


def argumentos():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--dir", default=str(RAIZ / "results" / "validacion"))
    ap.add_argument("--salida", default=str(RAIZ / "results" / "validacion_exacta.csv"))
    ap.add_argument("--tiempo", type=float, default=600.0, help="segundos de CP-SAT por instancia")
    ap.add_argument("--workers", type=int, default=WORKERS,
                    help="hilos de CP-SAT (por defecto todos los lógicos)")
    ap.add_argument("--reto", default=str(RETO))
    ap.add_argument("--ids", help="ids separados por comas (por defecto todas)")
    ap.add_argument("--formulaciones", default="basica,ajustada",
                    help="en orden; la siguiente sólo si la anterior no cierra")
    ap.add_argument("--actualizar", action="store_true",
                    help="conserva las filas de --salida que no se vuelven a correr")
    return ap


def main(argv=None):
    for flujo in (sys.stdout, sys.stderr):  # la consola de Windows (cp1252) no tiene ↔ ni ≠
        if hasattr(flujo, "reconfigure"):
            flujo.reconfigure(encoding="utf-8")
    a = argumentos().parse_args(argv)
    ids = a.ids.split(",") if a.ids else None
    return validar(a.dir, a.salida, tiempo=a.tiempo, workers=a.workers, reto=a.reto, ids=ids,
                   formulaciones=tuple(a.formulaciones.split(",")), actualizar=a.actualizar)


if __name__ == "__main__":
    sys.exit(main())
