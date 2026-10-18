import numpy as np
import pandas as pd

from paso2_compra import (cargar_instancia, escenarios_C, resolver_ple,
                          voceador, metricas)

RNG = np.random.default_rng(1)
C_COSTO = np.array([26, 26, 26, 26, 18, 21, 18, 18, 26.5, 26])


def sintetico(n=101):
    D = np.array([40, 100, 40, 40, 50, 40, 70, 60, 90, 30])
    X = RNG.integers(0, 60, size=(n, 10))
    return D, X


def test_C_es_D_menos_X():
    D, X = sintetico(5)
    C = escenarios_C(D, X)
    assert C.shape == (5, 10)
    assert (C == D - X).all()


def test_sin_presupuesto_ple_coincide_con_voceador():
    D, X = sintetico(101)
    C = escenarios_C(D, X)
    for pf in (2, 4, 10):
        q, obj = resolver_ple(C, C_COSTO, pf, presupuesto=None)
        qv = voceador(C, C_COSTO, pf)
        assert (q == qv).all()
        m = metricas(C, C_COSTO, qv, pf)
        assert abs(obj - m["objetivo"]) < 1e-6


def test_con_presupuesto_costo_menor_igual():
    D, X = sintetico(101)
    C = escenarios_C(D, X)
    q0, _ = resolver_ple(C, C_COSTO, 4, presupuesto=None)
    B = 0.6 * float(C_COSTO @ q0)
    q, obj = resolver_ple(C, C_COSTO, 4, presupuesto=B)
    assert float(C_COSTO @ q) <= B + 1e-6
    assert (q >= 0).all() and (q == np.round(q)).all()


def test_metricas_faltante_sobrante():
    C = np.array([[5, 0], [1, 3]])
    m = metricas(C, np.array([1.0, 1.0]), np.array([2, 2]), 2)
    assert m["faltante"] == (3 + 0 + 0 + 1) / 2
    assert m["sobrante"] == (0 + 2 + 1 + 0) / 2


def test_cargar_instancia(tmp_path):
    cols = ["omega", "T"] + [f"X_{j}" for j in range(10)]
    pd.DataFrame([[0, 1] + list(range(10))], columns=cols).to_csv(
        tmp_path / "escenarios_X_1ha.csv", index=False)
    X = cargar_instancia(tmp_path / "escenarios_X_1ha.csv")
    assert X.shape == (1, 10) and X[0, 9] == 9
