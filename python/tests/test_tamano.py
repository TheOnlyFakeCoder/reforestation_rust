import numpy as np
import pandas as pd
import pytest

import tamano

INST = {"1ha": dict(map_id=0, area_ha=1.0, n_pos=49, e=(30, 25, 20)),
        "map_27": dict(map_id=27, area_ha=1.28, n_pos=60, e=(40, 35, 30))}


@pytest.fixture
def origen(tmp_path):
    rng = np.random.default_rng(0)
    malla, comp, p3 = [], [], []
    for nombre, d in INST.items():
        malla.append(dict(instancia=nombre, map_id=d["map_id"], area_ha=d["area_ha"],
                          n_pos=d["n_pos"], d_tot=d["n_pos"],
                          pares_anillo_1=d["e"][0], pares_anillo_2=d["e"][1],
                          pares_anillo_3=d["e"][2]))
        D = rng.integers(3, 8, size=10)
        fila = dict(instancia=nombre, map_id=d["map_id"], area_ha=d["area_ha"],
                    d_tot=d["n_pos"])
        fila.update({f"D_{j}": int(D[j]) for j in range(10)})
        fila.update({f"Dnom_{j}": int(D[j]) for j in range(10)})
        comp.append(fila)
        p3.append(dict(instancia=nombre, omega=0, T=10, V_lib=d["n_pos"] - 10))
        p3.append(dict(instancia=nombre, omega=1, T=11, V_lib=d["n_pos"] - 11))
        X = rng.integers(0, 5, size=(20, 10))
        df = pd.DataFrame(X, columns=[f"X_{j}" for j in range(10)])
        df.insert(0, "T", 10)
        df.insert(0, "omega", range(20))
        df.to_csv(tmp_path / f"escenarios_X_{nombre}.csv", index=False)
    pd.DataFrame(malla).to_csv(tmp_path / "malla_resumen.csv", index=False)
    pd.DataFrame(comp).to_csv(tmp_path / "paso1_composicion.csv", index=False)
    pd.DataFrame(p3).to_csv(tmp_path / "paso3_escenarios.csv", index=False)
    return tmp_path


def fila(df, inst, modelo):
    return df[(df.instancia == inst) & (df.modelo == modelo)].iloc[0]


def test_tamano_paso1_lineal(origen):
    df = tamano.tamano_modelos(origen)
    assert list(df.columns) == ["instancia", "modelo", "variables", "tipo_variables",
                                "restricciones", "nota"]
    r = fila(df, "1ha", "paso1_lineal")
    E = 30 + 25 + 20
    assert r.variables == 490 + 100 * E
    assert r.restricciones == 49 + 10 + 20 + 3 * 100 * E
    assert fila(df, "1ha", "paso1_tabu").variables == 49


def test_paso2_y_paso3(origen):
    df = tamano.tamano_modelos(origen)
    r = fila(df, "map_27", "paso2_ple")
    assert 10 < r.variables <= 10 + 2 * 10 * 20
    assert "agrupa" in r.nota.lower()
    assert fila(df, "map_27", "paso3_por_escenario").variables == 50


def test_extensivo_mayor_que_suma(origen):
    df = tamano.tamano_modelos(origen)
    for inst in INST:
        ext = fila(df, inst, "extensivo_completo")
        suma = (fila(df, inst, "paso1_lineal").variables + fila(df, inst, "paso2_ple").variables
                + 1000 * fila(df, inst, "paso3_por_escenario").variables)
        assert ext.variables > suma
        assert "descomposición" in ext.nota
    E = 75
    ext = fila(df, "1ha", "extensivo_completo")
    assert ext.variables == (490 + 100 * E) + 10 + 1000 * (490 + 100 * E + 20)


def test_parametros_total():
    df = tamano.parametros()
    assert list(df.columns) == ["parametro", "simbolo", "cantidad", "fuente"]
    cant = df.set_index("simbolo").cantidad
    assert cant["W"] == 100
    assert cant["rasgos"] == 70
    assert df.iloc[-1].parametro == "TOTAL"
    assert df.iloc[-1].cantidad == df.iloc[:-1].cantidad.sum()


def test_problema_tiene_dnom(origen):
    df = tamano.problema(origen)
    assert list(df.instancia) == ["1ha", "map_27"]
    for c in ["especies", "plantas_por_ha", "D_tot", "n_pos"] + \
             [f"D_{j}" for j in range(10)] + [f"Dnom_{j}" for j in range(10)]:
        assert c in df.columns
    assert df.especies.eq(10).all()
    assert df.iloc[1].plantas_por_ha == pytest.approx(60 / 1.28)


def test_main_escribe_tres_csv(origen, tmp_path_factory):
    out = tmp_path_factory.mktemp("salida")
    tamano.main(origen=origen, destino=out)
    for n in ("tamano_modelo", "tamano_parametros", "tamano_problema"):
        assert (out / f"{n}.csv").exists()


def test_nota_dice_que_cuentan_las_restricciones(origen):
    df = tamano.tamano_modelos(origen)
    t = fila(df, "1ha", "paso1_tabu")
    assert t.restricciones == 30
    assert "10 definiciones" in t.nota and "20 cotas" in t.nota and "sin restricciones lineales" in t.nota
    p3 = fila(df, "1ha", "paso3_por_escenario")
    assert p3.restricciones == 10 and "10 cuotas" in p3.nota
    ext = fila(df, "1ha", "extensivo_completo")
    assert "1 restricción de presupuesto" in ext.nota
