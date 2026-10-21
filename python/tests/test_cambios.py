import pandas as pd

import cambios


def _dicts(zopt_a=100.0, zopt_d=99.0, d_a=5, d_d=5, z_iii=200.0, z_tabu=150.0, q_a=10, q_d=11):
    def g(z):
        f = {"instancia": ["1ha"] * 3, "gamma": [3, 3, 5], "semilla": [1, 2, 3], "z_opt": [z, z, 1e9]}
        f.update({f"D_{j}": [5] * 3 for j in range(10)})
        return pd.DataFrame(f)

    def comp(d):
        f = {"instancia": ["1ha"]}
        f.update({f"D_{j}": [d] for j in range(10)})
        return pd.DataFrame(f)

    def compra(q, c):
        f = {"instancia": ["1ha"] * 3, "epsilon": [0.1, 0.1, 0.2], "pi_factor": [2, 4, 2],
             "costo": [c, 0.0, 7.0]}
        f.update({f"q_{j}": [q, 0, 0] for j in range(10)})
        return pd.DataFrame(f)

    def pd3(zi, zt):
        return pd.DataFrame({"instancia": ["1ha"] * 2,
                             "variante": ["iii_serpentina_existentes", "tabu_con_existentes"],
                             "Z": [zi, zt]})

    antes = {"paso1_gamma_star": g(zopt_a), "paso1_composicion": comp(d_a),
             "paso3_resumen": pd.DataFrame({"instancia": ["1ha"], "Z_media": [50.0]}),
             "paso2_compra": compra(q_a, 1000.0), "practica_distribucion": pd3(z_iii, z_tabu)}
    despues = {"paso1_gamma_star": g(zopt_d), "paso1_composicion": comp(d_d),
               "paso3_resumen": pd.DataFrame({"instancia": ["1ha"], "Z_media": [50.0]}),
               "paso2_compra": compra(q_d, 1000.0), "practica_distribucion": pd3(z_iii, z_tabu)}
    return antes, despues


def test_tabla_cambios_z_opt():
    t = cambios.tabla_cambios(*_dicts())
    assert list(t.columns) == ["cifra", "instancia", "antes", "despues", "cambio_pct"]
    f = t[(t["cifra"] == "Z_opt paso 1") & (t["instancia"] == "1ha")].iloc[0]
    assert f["antes"] == 100.0 and f["despues"] == 99.0
    assert abs(f["cambio_pct"] - (-1.0)) < 1e-9


def test_tabla_cambios_reduccion_serpentina():
    a, d = _dicts(z_iii=200.0, z_tabu=150.0)
    t = cambios.tabla_cambios(a, d)
    f = t[t["cifra"] == "reducción vs serpentina iii"].iloc[0]
    assert f["despues"] == 25.0


def test_tabla_cambios_compra():
    a, d = _dicts(q_a=10, q_d=11)
    t = cambios.tabla_cambios(a, d)
    c = t[t["cifra"] == "costo paso 2"]
    assert len(c) == 1 and c.iloc[0]["despues"] == 1000.0  # sólo ε = 0.1 y π_factor = 2
    q = t[t["cifra"] == "q paso 2"].iloc[0]
    assert q["antes"].startswith("[10,") and q["despues"].startswith("[11,")


def test_d_star_sin_cambio():
    t = cambios.tabla_cambios(*_dicts(d_a=5, d_d=5))
    f = t[t["cifra"] == "D* paso 1"].iloc[0]
    assert f["antes"] == f["despues"]
    assert f["antes"].startswith("[") and f["antes"].endswith("]")
