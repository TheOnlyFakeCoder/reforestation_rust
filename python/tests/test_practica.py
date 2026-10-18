import numpy as np
import pandas as pd

from practica_compra import evaluar, mayor_residuo, main as main_compra
from markov import muertes, reposicion, vivas_esperadas

DELTA = np.array([42, 196, 42, 42, 49, 38, 73, 64, 86, 26])


def test_mayor_residuo_suma_total_y_proporcional():
    q = mayor_residuo(524, DELTA, 658)
    assert q.sum() == 524
    assert np.abs(q - 524 * DELTA / 658).max() < 1


def test_mayor_residuo_empates_al_indice_menor():
    q = mayor_residuo(1, [1, 1, 1], 3)
    assert list(q) == [1, 0, 0]


def test_evaluar_valores_a_mano():
    c = np.array([10.0, 20.0])
    s1 = np.array([0.5, 1.0])
    D = np.array([5, 5])
    X = np.array([[3, 7], [0, 0]])  # C = [2,-2], [5,5]
    q = np.array([4, 4])
    r = evaluar(q, D, X, c, s1, presupuesto=150.0, area=2.0)
    assert r["costo_ha"] == 120 / 2
    assert r["cabe"] is True
    # escenario 0: p=[2,0] faltante 0 sobrante 2+4=6 ; escenario 1: p=[4,4] faltante 1+1 sobrante 0
    assert abs(r["faltante_medio"] - 1.0) < 1e-12
    assert abs(r["sobrante_medio"] - 3.0) < 1e-12
    assert abs(r["vivas_1ano"] - 3.5) < 1e-12
    assert abs(r["supervivencia"] - 0.625) < 1e-12


def test_markov_formulas():
    s1, s2 = np.array([0.9]), np.array([0.8])
    m1, m2 = muertes(s1, s2)
    assert abs(m1[0] - 0.1) < 1e-12 and abs(m2[0] - (1 - 0.8 / 0.9)) < 1e-12
    v1, v2 = vivas_esperadas(np.array([100]), s1, s2)
    assert abs(v1[0] - 90) < 1e-9 and abs(v2[0] - 80) < 1e-9
    assert abs(reposicion(s1, s2)[0] - (0.1 + 0.1 + 0.01)) < 1e-12


def test_main_escribe_csv(tmp_path):
    from comun import RAIZ
    main_compra(resultados=tmp_path, instancias=["1ha"], origen=RAIZ / "results")
    df = pd.read_csv(tmp_path / "practica_compra.csv")
    assert sorted(df["opcion"]) == ["a", "b", "c", "d"]
