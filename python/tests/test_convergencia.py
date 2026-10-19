import pandas as pd
import pytest

from convergencia import criterio, cumple, referencia, main


def escribir(dirp, p1, p1b, p3, pr):
    pd.DataFrame(p1).to_csv(dirp / "paso1_gamma_star.csv", index=False)
    pd.DataFrame(p1b).to_csv(dirp / "paso1_barrido_gamma.csv", index=False)
    pd.DataFrame(p3).to_csv(dirp / "paso3_escenarios.csv", index=False)
    pd.DataFrame(pr).to_csv(dirp / "practica_distribucion.csv", index=False)


def datos(sm_a, sm_b):
    """sm_a / sm_b: lista de motivos para paso3 de las instancias a y b."""
    p1 = [dict(instancia="a", motivo_paro_paso1="sin_mejora", motivo_paro_opt="sin_mejora")]
    p1b = [dict(instancia="a", motivo_paro_paso1="sin_mejora", motivo_paro_opt="tope")]
    p3 = [dict(instancia="a", motivo_paro=m) for m in sm_a] + \
         [dict(instancia="b", motivo_paro=m) for m in sm_b]
    pr = [dict(instancia="a", motivo_paro="tope")] * 5  # no debe contar
    return p1, p1b, p3, pr


def test_cuenta_busquedas_sin_practica(tmp_path):
    escribir(tmp_path, *datos(["sin_mejora"] * 3, ["sin_mejora", "tope"]))
    t = criterio(tmp_path).set_index("instancia")
    assert t.loc["a", "busquedas"] == 2 + 2 + 3
    assert t.loc["a", "paros_tope"] == 1
    assert t.loc["b", "busquedas"] == 2 and t.loc["b", "paros_sin_mejora"] == 1
    assert t.loc["total", "busquedas"] == 9
    assert t.loc["total", "frac_sin_mejora"] == pytest.approx(7 / 9)


def test_cumple_y_no_cumple(tmp_path):
    escribir(tmp_path, *datos(["sin_mejora"] * 30, ["sin_mejora"] * 30))
    assert cumple(criterio(tmp_path))
    assert main([str(tmp_path)]) == 0
    escribir(tmp_path, *datos(["sin_mejora"] * 3, ["sin_mejora", "tope"]))
    assert not cumple(criterio(tmp_path))  # b al 50 %
    assert main([str(tmp_path)]) == 1


def test_no_cumple_por_total(tmp_path):
    # cada instancia >= 80 % pero total < 90 %
    p1, p1b, _, pr = datos([], [])
    p3 = [dict(instancia="a", motivo_paro=m) for m in ["sin_mejora"] * 8 + ["tope"] * 2] + \
         [dict(instancia="b", motivo_paro=m) for m in ["sin_mejora"] * 8 + ["tope"] * 2]
    escribir(tmp_path, p1, p1b, p3, pr)
    t = criterio(tmp_path)
    assert not cumple(t)


def fila(inst, z, t1=0.1, t2=0.2):
    return dict(instancia=inst, z_opt=z, tiempo_paso1_s=t1, tiempo_opt_s=t2)


def test_referencia_dif_rel(tmp_path):
    fin, ref = tmp_path / "fin", tmp_path / "ref"
    fin.mkdir(), ref.mkdir()
    pd.DataFrame([fila("m", 100.0), fila("m", 102.0), fila("n", 50.0)]).to_csv(
        fin / "paso1_gamma_star.csv", index=False)
    pd.DataFrame([fila("m", 100.0), fila("m", 100.0), fila("n", 50.0)]).to_csv(
        ref / "paso1_gamma_star.csv", index=False)
    params = dict(candidatos=64, tenure=15, iter_por_posicion=50,
                  sin_mejora_por_posicion=0)
    n_pos = {"m": 10, "n": 20}
    t = referencia(fin, ref, params, dict(params, iter_por_posicion=500,
                   sin_mejora_por_posicion=10), n_pos, max_iter=20000, max_sin_mejora=3000)
    m_fin = t[(t.instancia == "m") & (t.corrida == "final")].iloc[0]
    m_ref = t[(t.instancia == "m") & (t.corrida == "referencia")].iloc[0]
    assert m_fin.z_opt_media == pytest.approx(101.0)
    assert m_fin.dif_rel == pytest.approx(0.01)
    assert m_ref.dif_rel == pytest.approx(0.0)
    assert m_fin.max_iter_efectivo == 20000 and m_ref.max_iter_efectivo == 20000
    assert m_ref.max_sin_mejora_efectivo == 3000
    assert m_fin.tiempo_medio_s == pytest.approx(0.15)
    n_ref = t[(t.instancia == "n") & (t.corrida == "referencia")].iloc[0]
    assert n_ref.max_iter_efectivo == 20000  # max(20000, 500*20=10000)
