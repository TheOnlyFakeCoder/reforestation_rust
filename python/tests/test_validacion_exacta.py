import itertools
import json
import math

import numpy as np
import pandas as pd
import pytest

import validacion_exacta as ve
from comun import RAIZ

DIR_VAL = RAIZ / "results" / "validacion"
RETO = RAIZ / "target" / "release" / "reto.exe"
ACTIVAS = [0, 1, 2]


def _rejilla(columnas, filas):
    """Posiciones de una rejilla y aristas inventadas: anillo 0 horizontal y vertical,
    anillo 1 diagonal, anillo 2 a distancia 2 en la fila."""
    pos = [(c, f) for f in range(filas) for c in range(columnas)]
    idx = {p: i for i, p in enumerate(pos)}
    aristas = []
    for (c, f), u in idx.items():
        for dc, df, a in [(1, 0, 0), (0, 1, 0), (1, 1, 1), (-1, 1, 1), (2, 0, 2)]:
            w = idx.get((c + dc, f + df))
            if w is not None:
                aristas.append([min(u, w), max(u, w), a])
    aristas.sort()
    return pos, aristas


def _peso(semilla, especies=10):
    rng = np.random.default_rng(semilla)
    p = np.zeros((3, 10, 10))
    for a, h in enumerate([1.0, 0.6, 0.3]):
        m = rng.uniform(0, 1, (especies, especies))
        m = (m + m.T) / 2
        m[rng.uniform(0, 1, (especies, especies)) < 0.15] = 0.0
        m = np.triu(m) + np.triu(m, 1).T
        p[a, :especies, :especies] = m * h
    return p.tolist()


def sintetica(modelo, columnas=3, filas=3, fija=None, cuotas=None, m=None, banda=None,
              gamma=1.0, theta=None, semilla=1, especies=3):
    pos, aristas = _rejilla(columnas, filas)
    n = len(pos)
    fija = list(fija) if fija is not None else [-1] * n
    objetivo_constante = 0.0
    peso = _peso(semilla, especies)
    for u, w, a in aristas:
        if fija[u] >= 0 and fija[w] >= 0:
            objetivo_constante += peso[a][fija[u]][fija[w]]
    if modelo == "paso3":
        theta = [0.0] * 10
        bl = bu = [0] * 10
        libres, d_tot = None, None
    else:
        bl, bu = banda
        libres = sum(1 for j in range(10) if bl[j] < bu[j])
        d_tot = sum(1 for f in fija if f < 0)
        theta = theta or [0.0] * 10
    return {
        "id": f"s_{modelo}", "modelo": modelo, "columnas": columnas, "filas": filas,
        "disposicion": "Completa", "n_pos": n, "posiciones": [[float(c), float(f)] for c, f in pos],
        "aristas": aristas, "peso": peso, "fija": fija, "cuotas": cuotas, "m": m,
        "banda_l": list(bl), "banda_u": list(bu), "n_especies_libres": libres, "d_tot": d_tot,
        "theta": list(theta), "gamma": gamma, "semilla_existentes": 1,
        "objetivo_constante": objetivo_constante, "d_nominal": [0] * 10,
    }


def fuerza_bruta(inst):
    libres = [v for v in range(inst["n_pos"]) if inst["fija"][v] < 0]
    if inst["modelo"] == "paso3":
        opciones = [-1] + [j for j in range(10) if inst["cuotas"][j] > 0]
    else:
        opciones = [j for j in range(10) if inst["banda_u"][j] > 0]
    mejor, arg = math.inf, None
    for combo in itertools.product(opciones, repeat=len(libres)):
        phi = list(inst["fija"])
        for v, j in zip(libres, combo):
            phi[v] = j
        if ve.violaciones(inst, phi):
            continue
        o = ve.objetivo_python(inst, phi)
        if o < mejor:
            mejor, arg = o, phi
    return mejor, arg


CASOS = {
    "paso3_sin_huecos": lambda: sintetica("paso3", cuotas=[4, 3, 2] + [0] * 7, m=9),
    "paso3_con_huecos": lambda: sintetica("paso3", cuotas=[3, 2, 2] + [0] * 7, m=6, semilla=2),
    "paso3_una_fija": lambda: sintetica("paso3", fija=[-1, -1, -1, -1, 1, -1, -1, -1, -1],
                                        cuotas=[3, 2, 3] + [0] * 7, m=8, semilla=3),
    "paso3_fijas_adyacentes_y_huecos": lambda: sintetica(
        "paso3", fija=[2, 0, -1, -1, -1, -1, -1, -1, -1], cuotas=[2, 3, 1] + [0] * 7, m=5, semilla=4),
    "paso1": lambda: sintetica("paso1", banda=([2, 2, 1] + [0] * 7, [5, 4, 3] + [0] * 7), gamma=3.0,
                               theta=[1.4, 0.7, 2.9] + [0.0] * 7, semilla=5),
}


@pytest.fixture(scope="module")
def brutos():
    return {k: (f(), *fuerza_bruta(f())) for k, f in CASOS.items()}


def test_z_python_a_mano():
    inst = sintetica("paso3", columnas=3, filas=1, fija=[0, 1, -1], cuotas=[1] + [0] * 9, m=1)
    p = inst["peso"]
    # aristas: (0,1,0), (0,2,2), (1,2,0)
    assert inst["aristas"] == [[0, 1, 0], [0, 2, 2], [1, 2, 0]]
    assert ve.z_python(inst, [0, 1, -1]) == pytest.approx(p[0][0][1])
    assert ve.z_python(inst, [0, 1, 0]) == pytest.approx(p[0][0][1] + p[2][0][0] + p[0][1][0])
    assert ve.objetivo_python(inst, [0, 1, 0]) == pytest.approx(ve.z_python(inst, [0, 1, 0]))


def test_objetivo_python_paso1_es_gamma_z_mas_theta():
    inst = CASOS["paso1"]()
    phi = [0, 1, 2, 0, 1, 2, 0, 1, 0]
    z = ve.z_python(inst, phi)
    esperado = 3.0 * z + 4 * 1.4 + 3 * 0.7 + 2 * 2.9
    assert ve.objetivo_python(inst, phi) == pytest.approx(esperado, abs=1e-12)


def test_violaciones():
    inst = CASOS["paso3_una_fija"]()
    assert ve.violaciones(inst, [0, 0, 0, 1, 1, 2, 2, 2, 1]) == []
    assert ve.violaciones(inst, [0, 0, 0, 1, 2, 2, 2, 2, 1])  # existente cambiada
    assert ve.violaciones(inst, [0, 0, 0, 0, 1, 2, 2, 2, 1])  # cuota de 0 excedida
    assert ve.violaciones(inst, [0, 0, 0, 1, 1, 2, 2, -1, 1])  # M no alcanzada


@pytest.mark.parametrize("formulacion", ["basica", "ajustada"])
@pytest.mark.parametrize("caso", list(CASOS))
def test_cpsat_igual_a_fuerza_bruta(brutos, caso, formulacion):
    inst, z_bruto, _ = brutos[caso]
    r = ve.resolver(inst, tiempo=30, workers=2, formulacion=formulacion)
    assert r["optimo_probado"]
    assert ve.violaciones(inst, r["phi"]) == []
    assert r["objetivo"] == pytest.approx(z_bruto, abs=1e-9)
    assert r["objetivo"] == pytest.approx(ve.objetivo_python(inst, r["phi"]), abs=1e-12)
    assert r["z"] == pytest.approx(ve.z_python(inst, r["phi"]), abs=1e-12)
    assert r["cota"] <= r["objetivo"] + r["tolerancia_redondeo"] + 1e-12


def test_existente_existente_entra_como_constante(brutos):
    inst, z_bruto, phi = brutos["paso3_fijas_adyacentes_y_huecos"]
    assert inst["objetivo_constante"] > 0
    r = ve.resolver(inst, tiempo=30, workers=2)
    assert r["z"] >= inst["objetivo_constante"]
    assert r["cota"] == pytest.approx(r["objetivo"], abs=r["tolerancia_redondeo"] + 1e-9)


def test_escala_no_cambia_argmin(brutos):
    inst, z_bruto, _ = brutos["paso1"]
    for escala in (10_000, 1_000_000):
        r = ve.resolver(inst, tiempo=30, workers=2, escala=escala)
        assert ve.objetivo_python(inst, r["phi"]) == pytest.approx(z_bruto, abs=1e-9)


def test_hint_no_cambia_optimo(brutos):
    for caso in ["paso3_con_huecos", "paso1"]:
        inst, z_bruto, phi_opt = brutos[caso]
        libres = [v for v in range(inst["n_pos"]) if inst["fija"][v] < 0]
        # una solución factible no óptima: la peor de unas cuantas permutaciones del óptimo
        rng = np.random.default_rng(0)
        peor, phi_malo = -math.inf, None
        for _ in range(50):
            phi = list(phi_opt)
            vals = [phi[v] for v in libres]
            rng.shuffle(vals)
            for v, j in zip(libres, vals):
                phi[v] = j
            o = ve.objetivo_python(inst, phi)
            if o > peor:
                peor, phi_malo = o, phi
        assert peor > z_bruto + 1e-9
        for hint in (None, phi_malo, phi_opt):
            r = ve.resolver(inst, tiempo=30, workers=2, hint=hint)
            assert r["hint"] == (hint is not None)
            assert r["optimo_probado"]
            assert r["objetivo"] == pytest.approx(z_bruto, abs=1e-9)


def test_hint_entra_al_modelo():
    inst = CASOS["paso3_con_huecos"]()
    _, phi = fuerza_bruta(inst)
    modelo, x = ve.modelo_cpsat(inst, hint=phi)
    proto = modelo.Proto()
    assert len(proto.solution_hint.vars) > 0
    hint = dict(zip(proto.solution_hint.vars, proto.solution_hint.values))
    for v, por_especie in x.items():
        for j, var in por_especie.items():
            assert hint[var.Index()] == int(phi[v] == j)


def test_modelo_cpsat_variables_por_libre():
    inst = CASOS["paso3_una_fija"]()
    modelo, x = ve.modelo_cpsat(inst)
    assert sorted(x) == [0, 1, 2, 3, 5, 6, 7, 8]
    assert all(sorted(x[v]) == ACTIVAS for v in x)
    _, xa = ve.modelo_cpsat(inst, formulacion="ajustada")
    assert sorted(xa) == sorted(x)


def test_ajustada_tiene_y_para_todos_los_pares():
    inst = CASOS["paso1"]()
    mb, _ = ve.modelo_cpsat(inst, formulacion="basica")
    ma, _ = ve.modelo_cpsat(inst, formulacion="ajustada")
    n_x = 9 * 3
    n_ff = len(inst["aristas"])
    assert len(ma.Proto().variables) == n_x + 9 * n_ff
    assert len(mb.Proto().variables) < len(ma.Proto().variables)


def test_error_redondeo_usa_gamma_por_peso(brutos):
    inst = brutos["paso1"][0]
    e = 10_000
    g = inst["gamma"]
    coefs = [g * inst["peso"][a][j][k] for _, _, a in inst["aristas"] for j in ACTIVAS for k in ACTIVAS]
    coefs += [inst["theta"][j] for j in ACTIVAS]
    esperado = max(abs(round(c * e) / e - c) for c in coefs)
    r = ve.resolver(inst, tiempo=30, workers=2)
    assert r["error_redondeo_max"] == pytest.approx(esperado, abs=1e-15)
    otro = max([abs(g * round(inst["peso"][a][j][k] * e) / e - g * inst["peso"][a][j][k])
                for _, _, a in inst["aristas"] for j in ACTIVAS for k in ACTIVAS])
    assert otro != pytest.approx(esperado, abs=1e-12)
    assert r["error_redondeo_max"] <= 0.5 / e


def _grande():
    """10×10 con 10 especies: CP-SAT no la cierra en una fracción de segundo."""
    cuotas = [12, 11, 11, 10, 10, 10, 9, 9, 9, 9]
    return sintetica("paso3", columnas=10, filas=10, cuotas=cuotas, m=100, semilla=7, especies=10)


def _tabu_falso(inst, n=3):
    """Asignaciones factibles (no óptimas) con z y objetivo exactos, como `tabu_<id>.csv`."""
    libres = [v for v in range(inst["n_pos"]) if inst["fija"][v] < 0]
    base = []
    for j, q in enumerate(inst["cuotas"]):
        base += [j] * q
    base = base[:inst["m"]] + [-1] * (len(libres) - inst["m"])
    rng = np.random.default_rng(3)
    filas = []
    for k in range(n):
        vals = list(base)
        rng.shuffle(vals)
        phi = list(inst["fija"])
        for v, j in zip(libres, vals):
            phi[v] = j
        filas.append({"semilla": 100 + k, "z": ve.z_python(inst, phi),
                      "objetivo": ve.objetivo_python(inst, phi), "tiempo_s": 0.5 + k,
                      "iteraciones": 10, "motivo_paro": "sin_mejora",
                      "phi": ";".join(str(c) for c in phi)})
    return pd.DataFrame(filas)


def test_no_optimo_se_reporta(tmp_path):
    inst = _grande()
    tabu = _tabu_falso(inst)
    phi_ts = ve.leer_phi(tabu.loc[tabu["objetivo"].idxmin(), "phi"])
    r = ve.resolver(inst, tiempo=0.3, workers=2, hint=phi_ts)
    assert not r["optimo_probado"]
    fila = ve.fila_resultado(inst, tabu, r)
    salida = tmp_path / "v.csv"
    ve.escribir_csv([fila], salida)
    df = pd.read_csv(salida)
    assert list(df.columns[:24]) == ve.COLUMNAS[:24]
    assert len(df) == 1 and not bool(df["optimo_probado"].iloc[0])
    cota = df["cota"].iloc[0]
    assert np.isfinite(cota)
    # en tan poco tiempo CP-SAT puede no tener incumbente (UNKNOWN): la fila se escribe igual
    assert df["estado"].iloc[0] in ("FEASIBLE", "UNKNOWN")
    assert np.isfinite(df["z_opt"].iloc[0]) == (df["estado"].iloc[0] == "FEASIBLE")
    if cota > 0:
        assert df["brecha_mejor"].iloc[0] == pytest.approx((tabu["objetivo"].min() - cota) / cota)
        assert df["brecha_media"].iloc[0] == pytest.approx((tabu["objetivo"].mean() - cota) / cota)
    else:  # cota trivial: brecha relativa no acotada
        assert df["brecha_mejor"].iloc[0] == math.inf and df["brecha_media"].iloc[0] == math.inf
    assert bool(df["hint"].iloc[0])


def test_brecha_contra_cota_no_positiva_es_infinita():
    assert ve.brecha(5.0, 0.0) == math.inf
    assert ve.brecha(5.0, 4.0) == pytest.approx(0.25)
    assert ve.brecha(4.0, 4.0) == 0.0
    assert math.isnan(ve.brecha(5.0, math.nan))


def test_fila_resultado_campos(brutos):
    inst, z_bruto, phi = brutos["paso1"]
    tabu = pd.DataFrame({"semilla": [1, 2, 3], "z": [0.0] * 3, "objetivo": [0.0] * 3,
                         "tiempo_s": [1.0, 2.0, 3.0], "iteraciones": [5] * 3,
                         "motivo_paro": ["sin_mejora"] * 3, "phi": [""] * 3})
    otro = list(phi)
    otro[0], otro[1] = otro[1], otro[0]
    if ve.objetivo_python(inst, otro) == pytest.approx(z_bruto):
        otro[0], otro[2] = otro[2], otro[0]
    for i, p in enumerate([phi, otro, phi]):
        tabu.loc[i, "phi"] = ";".join(map(str, p))
        tabu.loc[i, "z"] = ve.z_python(inst, p)
        tabu.loc[i, "objetivo"] = ve.objetivo_python(inst, p)
    r = ve.resolver(inst, tiempo=30, workers=2, hint=phi)
    f = ve.fila_resultado(inst, tabu, r)
    assert f["modelo"] == "paso1" and f["formulacion"] == "basica" and f["hint"] is True
    assert f["n_pos"] == 9 and f["existentes"] == 0 and f["aristas"] == len(inst["aristas"])
    assert f["n_especies_libres"] == 3
    assert f["z_opt"] == pytest.approx(z_bruto) and f["optimo_probado"]
    assert f["z_ts_mejor"] == pytest.approx(z_bruto)
    assert f["z_ts_media"] == pytest.approx(tabu["objetivo"].mean())
    assert f["z_ts_sd"] == pytest.approx(tabu["objetivo"].std(ddof=1))
    assert f["tiempo_ts_medio_s"] == pytest.approx(2.0)
    assert f["brecha_mejor"] == pytest.approx(0.0, abs=1e-12)
    assert f["brecha_media"] == pytest.approx((tabu["objetivo"].mean() - z_bruto) / z_bruto)
    assert f["frac_semillas_optimo"] == pytest.approx(2 / 3)
    assert f["z_comp_opt"] == pytest.approx(ve.z_python(inst, r["phi"]))
    assert f["z_comp_ts_mejor"] == pytest.approx(ve.z_python(inst, phi))
    zc = f["z_comp_opt"]
    assert f["brecha_z"] == pytest.approx((3 * f["z_comp_ts_mejor"] - 3 * zc) / (3 * zc))


def test_tabu_real_coincide_con_rust():
    rutas = sorted(DIR_VAL.glob("inst_*.json"))
    assert len(rutas) == 10
    for ruta in rutas:
        inst = ve.leer_instancia(ruta)
        tabu = ve.leer_tabu(DIR_VAL / ruta.name.replace("inst_", "tabu_").replace(".json", ".csv"))
        assert len(tabu) == 10
        assert ve.verificar_tabu(inst, tabu) == []


def test_verificar_tabu_detecta_discrepancia():
    inst = _grande()
    tabu = _tabu_falso(inst)
    tabu.loc[1, "z"] += 1e-4
    d = ve.verificar_tabu(inst, tabu)
    assert len(d) == 1 and "z" in d[0]


@pytest.mark.skipif(not RETO.exists(), reason="falta target/release/reto.exe")
def test_evaluar_val_rust_coincide_con_python_con_existentes(tmp_path):
    ruta = DIR_VAL / "inst_p3_c.json"
    inst = ve.leer_instancia(ruta)
    assert inst["objetivo_constante"] > 0
    tabu = ve.leer_tabu(DIR_VAL / "tabu_p3_c.csv")
    phis = [ve.leer_phi(t) for t in tabu["phi"]]
    res = ve.evaluar_rust(RETO, ruta, phis, tmp_path)
    for phi, (z, o, avisos) in zip(phis, res):
        assert avisos == ""
        assert abs(z - ve.z_python(inst, phi)) <= 1e-6
        assert abs(o - ve.objetivo_python(inst, phi)) <= 1e-6


def _carpeta(tmp_path, inst, tabu, nombre):
    inst = dict(inst, id=nombre)
    d = tmp_path / "val"
    d.mkdir(exist_ok=True)
    (d / f"inst_{nombre}.json").write_text(json.dumps(inst), encoding="utf-8")
    tabu.to_csv(d / f"tabu_{nombre}.csv", index=False)
    return d


def test_validar_discrepancia_sale_con_codigo_2(tmp_path):
    inst = _grande()
    tabu = _tabu_falso(inst)
    tabu.loc[0, "z"] += 0.01
    d = _carpeta(tmp_path, inst, tabu, "x")
    salida = tmp_path / "v.csv"
    codigo = ve.validar(d, salida, tiempo=0.2, workers=2, reto=RETO)
    assert codigo == 2
    disc = salida.with_name(salida.stem + "_discrepancias.txt")
    assert disc.exists() and "x" in disc.read_text(encoding="utf-8")


@pytest.mark.skipif(not RETO.exists(), reason="falta target/release/reto.exe")
def test_validar_corre_ajustada_solo_si_basica_no_cierra(tmp_path):
    chica = CASOS["paso3_con_huecos"]()
    grande = _grande()
    d = _carpeta(tmp_path, chica, _tabu_falso(chica), "a_chica")
    _carpeta(tmp_path, grande, _tabu_falso(grande), "b_grande")
    salida = tmp_path / "v.csv"
    codigo = ve.validar(d, salida, tiempo=0.3, workers=2, reto=RETO)
    assert codigo == 0
    df = pd.read_csv(salida)
    assert list(df.columns) == ve.COLUMNAS
    assert list(zip(df["id"], df["formulacion"])) == [
        ("a_chica", "basica"), ("b_grande", "basica"), ("b_grande", "ajustada")]
    assert bool(df["optimo_probado"].iloc[0])
    assert not bool(df["optimo_probado"].iloc[1])
    assert df["hint"].all()
