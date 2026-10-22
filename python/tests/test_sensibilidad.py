import json
import subprocess
from pathlib import Path

import numpy as np
import pandas as pd
import pytest

from comun import RAIZ, leer_config
import sensibilidad

RETO = RAIZ / "target" / "release" / "reto.exe"


def test_config_variante():
    base = leer_config()
    assert base["escenarios_paso3_por_instancia"] != {}
    cfg = sensibilidad.config_variante(base, {"kappa": 0.4})
    assert cfg["kappa"] == 0.4
    assert cfg["escenarios_paso3_por_instancia"] == {}
    assert base["kappa"] == 0.6 and base["escenarios_paso3_por_instancia"] != {}
    assert set(sensibilidad.VARIANTES) == {"base", "kappa_0.4", "kappa_0.8", "radios_0.75",
                                           "radios_1.25", "m_0", "m_0.5", "m_2", "s2"}
    assert sensibilidad.VARIANTES["s2"] == {"usar_s2": True}
    assert sensibilidad.VARIANTES["base"] == {}


@pytest.mark.skipif(not RETO.exists(), reason="falta reto.exe")
def test_config_variante_m0_valida(tmp_path):
    cfg = sensibilidad.config_variante(leer_config(), sensibilidad.VARIANTES["m_0"])
    (tmp_path / "config.json").write_text(json.dumps(cfg), encoding="utf-8")
    tabu = ('tabu={"candidatos":16,"tenure":5,"max_iter":200,"max_sin_mejora":50,'
            '"prob_intercambio":0.5,"iter_por_posicion":1,"sin_mejora_por_posicion":0}')
    r = subprocess.run([str(RETO), "paso1", "--instancias", "1ha", "--solo-gamma-star",
                        "--datos", str(RAIZ / "data"), "--config", str(tmp_path / "config.json"),
                        "--resultados", str(tmp_path), "--set", "n_semillas=1", "--set", tabu],
                       capture_output=True, text=True, cwd=RAIZ)
    assert r.returncode == 0, r.stderr


def _escribir(carpeta, zopt, z3, costo, con_paso3=True):
    carpeta.mkdir(parents=True)
    D = {f"D_{j}": 10 + j for j in range(10)}
    insts = ["1ha", "map_27", "map_21"]
    pd.DataFrame([{"instancia": i, **D} for i in insts]).to_csv(
        carpeta / "paso1_composicion.csv", index=False)
    filas = []
    for i in insts:
        for s, dz in enumerate([-1, 1]):
            filas.append({"instancia": i, "gamma": 3, "semilla": s, **D,
                          "suma_theta_d": 100.0, "z_opt": zopt + dz})
    pd.DataFrame(filas).to_csv(carpeta / "paso1_gamma_star.csv", index=False)
    if not con_paso3:
        return
    pd.DataFrame([{"instancia": i, "epsilon": 0.1, "pi_factor": 2, "costo": costo,
                   "faltante_esperado": 5.0} for i in ["1ha", "map_27"]]).to_csv(
        carpeta / "paso2_compra.csv", index=False)
    pd.DataFrame([{"instancia": i, "Z_media": z3, "Z_sd": 2.0}
                  for i in ["1ha", "map_27"]]).to_csv(carpeta / "paso3_resumen.csv", index=False)
    pd.DataFrame([{"instancia": i, "variante": v, "Z": z}
                  for i in ["1ha", "map_27"]
                  for v, z in [("iii_serpentina_existentes", 900.0),
                               ("tabu_con_existentes", 600.0),
                               ("dsatur_con_existentes", 700.0)]]).to_csv(
        carpeta / "practica_distribucion.csv", index=False)


def test_resumir(tmp_path):
    _escribir(tmp_path / "base", 500.0, 600.0, 12000.0)
    _escribir(tmp_path / "kappa_0.4", 550.0, 660.0, 12100.0)
    df = sensibilidad.resumir(tmp_path)
    k = df[(df.variante == "kappa_0.4") & (df.instancia == "1ha")].iloc[0]
    assert k["z_opt_media"] == pytest.approx(550.0)
    assert k["z_opt_sd"] == pytest.approx(np.std([549, 551], ddof=1))
    assert k["cambio_z_paso3_pct"] == pytest.approx(10.0)
    assert k["cambio_z_opt_pct"] == pytest.approx(10.0)
    assert k["z_practica_iii"] == 900.0 and k["z_tabu_con"] == 600.0 and k["z_dsatur_con"] == 700.0
    assert k["costo_compra"] == 12100.0 and k["faltante_esperado"] == 5.0
    m = df[(df.variante == "kappa_0.4") & (df.instancia == "map_21")].iloc[0]
    assert m["z_opt_media"] == pytest.approx(550.0)
    assert np.isnan(m["z_paso3_media"]) and np.isnan(m["cambio_z_paso3_pct"])
    assert (df[df.variante == "base"]["cambio_z_opt_pct"] == 0).all()
    assert len(df) == 6
    assert list(df.columns[:2]) == ["variante", "instancia"]


def test_resumir_variante_sin_paso3(tmp_path):
    _escribir(tmp_path / "base", 500.0, 600.0, 12000.0)
    _escribir(tmp_path / "m_0", 400.0, 0, 0, con_paso3=False)
    df = sensibilidad.resumir(tmp_path)
    assert len(df) == 6
    assert df[df.variante == "m_0"]["z_paso3_media"].isna().all()


def test_reanuda_variante_hecha(tmp_path):
    carpeta = tmp_path / "base"
    _escribir(carpeta, 500.0, 600.0, 12000.0)
    cfg = sensibilidad.config_variante(leer_config(), {})
    (carpeta / "config.json").write_text(json.dumps(cfg), encoding="utf-8")

    def falso(*a, **k):
        raise AssertionError("no debe ejecutarse")
    sensibilidad.correr_variante("base", carpeta, Path("reto.exe"), ejecutar=falso)


def test_correr_variante_ordena_los_pasos(tmp_path, monkeypatch):
    llamadas = []
    monkeypatch.setattr(sensibilidad.paso2_compra, "correr",
                        lambda resultados=None, salida=None: llamadas.append(("paso2", resultados)))

    def falso(cmd, **k):
        llamadas.append(cmd[1])
        return subprocess.CompletedProcess(cmd, 0)
    carpeta = tmp_path / "kappa_0.4"
    sensibilidad.correr_variante("kappa_0.4", carpeta, Path("reto.exe"), ejecutar=falso,
                                 mc_params=None)
    assert [l if isinstance(l, str) else l[0] for l in llamadas] == [
        "malla", "paso1", "montecarlo", "paso2", "paso3", "practica"]
    assert json.loads((carpeta / "config.json").read_text())["kappa"] == 0.4
