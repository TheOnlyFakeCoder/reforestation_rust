import numpy as np
import pandas as pd

import markov
from comun import RAIZ


def test_compra_proyectada_coincide_con_practica_d(tmp_path):
    df = markov.main(resultados=tmp_path, instancias=["1ha"])
    assert {"D_completo", "compra_d", "compra_nominal"} <= set(df["base"])
    assert (df[df["composicion"] == "D*"]["base"].isin(["D_completo", "compra_d"])).all()
    pc = pd.read_csv(RAIZ / "results" / "practica_compra.csv")
    vivas_d = pc[(pc["instancia"] == "1ha") & (pc["opcion"] == "d")]["vivas_1ano"].iloc[0]
    mk = df[(df["base"] == "compra_d")]["vivas_1ano"].sum()
    assert abs(mk - vivas_d) < 0.01


def test_filas_d_completo_intactas(tmp_path):
    df = markov.main(resultados=tmp_path, instancias=["1ha"])
    full = df[df["base"] == "D_completo"]
    assert len(full) == 20
    assert abs(full[full["composicion"] == "D*"]["plantas"].sum() - 658) < 1e-6
    p = df[df["base"] == "compra_d"]
    assert p["plantas"].sum() < 658
    assert np.allclose(p["reposicion_esperada"], p["plantas"] * p["R_j"])
