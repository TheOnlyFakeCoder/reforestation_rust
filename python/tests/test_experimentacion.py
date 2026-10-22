import numpy as np
import pandas as pd
import pytest

import experimentacion

J = 10
NIVELES = [f"Z_nivel{k}" for k in range(1, 6)] + ["Z_otro"]


def _escribir(o, area=2.0, D=None, q=None, X0=(2, 4, 6, 8), Z=(100.0, 110.0, 120.0, 130.0)):
    D = D or [10] * J
    q = q or [5] * J
    comp = dict(instancia="1ha", map_id=0, area_ha=area, d_tot=sum(D))
    comp.update({f"D_{j}": D[j] for j in range(J)})
    pd.DataFrame([comp]).to_csv(o / "paso1_composicion.csv", index=False)
    filas = []
    for eps, pi, qq in [(0.1, 2, q), (0.0, 2, [99] * J), (0.1, 4, [77] * J)]:
        f = dict(instancia="1ha", map_id=0, epsilon=eps, pi_factor=pi)
        f.update({f"q_{j}": qq[j] for j in range(J)})
        f["costo"] = 1.0
        filas.append(f)
    pd.DataFrame(filas).to_csv(o / "paso2_compra.csv", index=False)
    X = np.zeros((len(X0), J), dtype=int)
    X[:, 0] = X0
    X[:, 1] = 3
    df = pd.DataFrame(X, columns=[f"X_{j}" for j in range(J)])
    df.insert(0, "T", X.sum(axis=1))
    df.insert(0, "omega", range(len(X0)))
    df.to_csv(o / "escenarios_X_1ha.csv", index=False)
    p3 = pd.DataFrame({"instancia": "1ha", "map_id": 0, "omega": range(len(Z)), "Z": Z})
    for k, c in enumerate(NIVELES):
        p3[c] = [z * (k + 1) / 21 for z in Z]
    p3.to_csv(o / "paso3_escenarios.csv", index=False)


def _fila(t, magnitud, especie=None):
    s = t[t["magnitud"] == magnitud]
    s = s[s["especie"].isna()] if especie is None else s[s["especie"] == especie]
    assert len(s) == 1, (magnitud, especie)
    return s.iloc[0]


def test_experimentacion_ic(tmp_path):
    _escribir(tmp_path)
    t = experimentacion.tabla(tmp_path, instancias=("1ha",))
    assert list(t.columns) == ["instancia", "magnitud", "especie", "n", "media_ha", "sd_ha",
                               "ic95_inf", "ic95_sup"]
    # C_0 = 10 − X_0 = (8, 6, 4, 2); área 2 ha
    c = _fila(t, "C", 0)
    sd = np.std([8, 6, 4, 2], ddof=1) / 2
    assert c["n"] == 4
    assert c["media_ha"] == pytest.approx(2.5)
    assert c["sd_ha"] == pytest.approx(sd)
    assert c["ic95_inf"] == pytest.approx(2.5 - 1.96 * sd / 2)
    assert c["ic95_sup"] == pytest.approx(2.5 + 1.96 * sd / 2)
    x = _fila(t, "X", 0)
    assert x["media_ha"] == pytest.approx(2.5)
    tt = _fila(t, "T")
    assert tt["media_ha"] == pytest.approx((5 + 3) / 2)  # T = X_0 + X_1, X_1 = 3
    z = _fila(t, "Z")
    sdz = np.std([100, 110, 120, 130], ddof=1) / 2
    assert z["media_ha"] == pytest.approx(57.5)
    assert z["ic95_sup"] == pytest.approx(57.5 + 1.96 * sdz / 2)
    for k, niv in enumerate(NIVELES):
        assert _fila(t, niv)["media_ha"] == pytest.approx(115 * (k + 1) / 21 / 2)
    q = _fila(t, "q", 3)
    assert q["media_ha"] == pytest.approx(2.5) and q["sd_ha"] == 0  # q = 5 (ε = 0.1, π = 2)
    assert q["ic95_inf"] == q["ic95_sup"] == pytest.approx(2.5)
    assert set(t["magnitud"]) == {"T", "X", "C", "n", "q", "Z", *NIVELES}
    assert len(t) == 2 + 4 * J + len(NIVELES)


def test_n_j_es_min_q_c(tmp_path):
    # q_0 = 5: C_0 = (8, 6, 4, 2) -> n_0 = (5, 5, 4, 2); especie 1: D = 2 < X = 3 -> C = n = 0
    D = [10, 2] + [10] * (J - 2)
    _escribir(tmp_path, area=1.0, D=D)
    t = experimentacion.tabla(tmp_path, instancias=("1ha",))
    assert _fila(t, "n", 0)["media_ha"] == pytest.approx(4.0)
    assert _fila(t, "C", 1)["media_ha"] == 0 and _fila(t, "n", 1)["media_ha"] == 0
    assert _fila(t, "n", 5)["media_ha"] == pytest.approx(5)  # C_5 = 10 > q_5 = 5


def test_main_escribe_csv(tmp_path):
    _escribir(tmp_path)
    ruta = experimentacion.main(origen=tmp_path, instancias=("1ha",))
    assert ruta == tmp_path / "experimentacion.csv"
    t = pd.read_csv(ruta)
    assert t.loc[t["magnitud"] == "Z", "especie"].isna().all()
