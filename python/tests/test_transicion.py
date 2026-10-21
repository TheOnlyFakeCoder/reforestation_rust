import numpy as np
import pandas as pd
import pytest

import transicion as tr

SENO_60 = np.sqrt(3) / 2


def malla_rect(columnas, filas, d=1.0):
    """Réplica de Malla::rectangular(columnas, filas, Completa, d) de Rust."""
    h = d * SENO_60
    reg = []
    for fila in range(filas):
        x0 = d / 2 if fila % 2 == 0 else d
        for col in range(columnas):
            reg.append((len(reg), fila, col, x0 + col * d, h / 2 + fila * h))
    return pd.DataFrame(reg, columns=["pos", "fila", "col", "x", "y"])


def caso():
    """Mismo caso fijo que src/transicion.rs: phi = [1,2,1,0,3,2], existente en pos 4."""
    p = malla_rect(3, 2)
    p["especie"] = [0, 1, 0, -1, 2, 1]
    p["existente"] = [0, 0, 0, 0, 1, 0]
    return p


def test_orden_serpentina_filas_alternas():
    p = malla_rect(3, 2)
    p["especie"] = [0, 1, 2, 3, 4, 5]
    p["existente"] = 0
    o = tr.orden_serpentina(p)
    assert list(o["pos"]) == [0, 1, 2, 5, 4, 3]
    assert list(o["paso"]) == [1, 2, 3, 4, 5, 6]
    assert "existente" not in o.columns


def test_orden_salta_existentes_y_vacias():
    o = tr.orden_serpentina(caso())
    assert list(o["pos"]) == [0, 1, 2, 5]
    assert list(o["especie"]) == [0, 1, 0, 1]
    assert list(o.columns) == ["paso", "pos", "fila", "col", "x", "y", "especie"]


def test_matriz_orden_conteos():
    m = tr.matriz_orden(tr.orden_serpentina(caso()))
    c = m.pivot(index="de", columns="a", values="conteo").to_numpy()
    esperado = np.zeros((10, 10), int)
    esperado[0, 1] = 2
    esperado[1, 0] = 1
    assert (c == esperado).all()
    assert list(m.columns) == ["de", "a", "conteo", "n", "prob"]
    assert len(m) == 100


def test_matriz_vecindad_igual_que_rust():
    m = tr.matriz_vecindad(caso(), d=1.0)
    c = m.pivot(index="de", columns="a", values="conteo").to_numpy()
    esperado = np.zeros((10, 10), int)
    for a, b, v in [(0, 1, 3), (1, 2, 2), (0, 2, 1)]:
        esperado[a, b] = esperado[b, a] = v
    assert (c == esperado).all()
    assert c.sum() == 12


def test_fila_sin_ocurrencias():
    m = tr.matriz_vecindad(caso(), d=1.0)
    f = m[m["de"] == 5]
    assert (f["n"] == 0).all() and (f["prob"] == 0).all()
    assert not m["prob"].isna().any()


def test_prob_suma_uno():
    m = tr.matriz_vecindad(caso(), d=1.0)
    s = m.groupby("de")["prob"].sum()
    for j, v in s.items():
        assert v == pytest.approx(1.0 if j in (0, 1, 2) else 0.0)


def test_estacionaria_igual_a_n_relativo():
    m = tr.matriz_vecindad(caso(), d=1.0)
    pi = tr.estacionaria(m)
    n = m.groupby("de")["n"].first()
    n = n[n > 0]
    for j in n.index:
        assert pi[j] == pytest.approx(n[j] / n.sum(), abs=1e-9)


def test_estacionaria_con_especie_ausente():
    m = tr.matriz_vecindad(caso(), d=1.0)
    pi = tr.estacionaria(m)
    assert list(pi.index) == list(range(10))
    assert pi[3:].eq(0).all()
    assert pi.sum() == pytest.approx(1.0)
    assert not pi.isna().any()


def test_estacionaria_cerca_de_composicion():
    p = malla_rect(20, 20)
    rng = np.random.default_rng(7)
    esp = np.array([0] * 40 + [1] * 120 + [2] * 240)
    rng.shuffle(esp)
    p["especie"] = esp
    p["existente"] = 0
    pi = tr.estacionaria(tr.matriz_vecindad(p, d=1.0))
    for j, f in [(0, 0.1), (1, 0.3), (2, 0.6)]:
        assert abs(pi[j] - f) <= 0.02


def test_main_escribe_salidas(tmp_path):
    origen, salida = tmp_path / "o", tmp_path / "s"
    origen.mkdir()
    p = caso()
    p.to_csv(origen / "paso3_patron_t_omega0.csv", index=False)
    p.to_csv(origen / "practica_patron_t_iii.csv", index=False)
    filas = []
    for tipo in ("vecindad", "orden"):
        for j in range(10):
            for k in range(10):
                filas.append([tipo, j, k, int(j == 0 and k == 1), int(j == 0), float(j == 0 and k == 1)])
    pd.DataFrame(filas, columns=["tipo", "de", "a", "conteo", "n", "prob"]).to_csv(
        origen / "paso3_transicion_t.csv", index=False)
    pd.DataFrame({"especie": range(10), "plantas": [2, 2, 1] + [0] * 7,
                  "fraccion": [.4, .4, .2] + [0] * 7}).to_csv(origen / "paso3_plantas_t.csv", index=False)
    tr.main(origen=origen, salida=salida, instancias=("t",))
    assert len(pd.read_csv(salida / "orden_plantacion_t.csv")) == 4
    t = pd.read_csv(salida / "transicion.csv")
    assert list(t.columns) == ["instancia", "patron", "alcance", "tipo", "de", "a", "conteo", "n", "prob"]
    assert set(t["alcance"]) == {"todos_los_escenarios", "omega0"}
    assert set(t["patron"]) == {"tabu_paso3", "practica_iii"}
    e = pd.read_csv(salida / "transicion_estacionaria.csv")
    assert list(e.columns) == ["instancia", "patron", "alcance", "especie", "pi_estacionaria",
                               "composicion_plantada"]
    # omega0: composición del patrón incluye la planta existente (pos 4, especie 2)
    r = e[(e["alcance"] == "omega0") & (e["patron"] == "tabu_paso3")].set_index("especie")
    assert r.loc[2, "composicion_plantada"] == pytest.approx(0.2)
