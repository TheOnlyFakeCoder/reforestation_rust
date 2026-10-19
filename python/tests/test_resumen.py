import numpy as np
import pandas as pd

import resumen
from comun import RAIZ


def test_resumen_real_tiene_secciones_y_numeros(tmp_path):
    ruta = resumen.main(origen=RAIZ / "results", salida=tmp_path / "RESUMEN.md")
    texto = ruta.read_text(encoding="utf-8")
    for titulo in ["## Malla", "## Ajuste de distribuciones", "## Paso 1", "## Monte Carlo",
                   "## Paso 2", "## Paso 3", "## Práctica actual", "## Figuras"]:
        assert titulo in texto
    assert "| 1ha |" in texto and "658" in texto
    assert "presupuesto_corto" in texto
    assert "γ* = 3" in texto
    assert "../figures/mapa_1ha.png" in texto


def test_tabla_md_formato():
    df = pd.DataFrame({"a": [1, 2], "b": [0.12345678, 3.0]})
    t = resumen.tabla_md(df)
    lineas = t.splitlines()
    assert lineas[0] == "| a | b |"
    assert lineas[1] == "| --- | --- |"
    assert lineas[2] == "| 1 | 0.1235 |"
    assert lineas[3] == "| 2 | 3 |"


def test_resumen_correcciones_finales(tmp_path):
    texto = resumen.main(origen=RAIZ / "results", salida=tmp_path / "R.md").read_text(encoding="utf-8")
    assert "unas 12 posiciones" not in texto
    for frase in ["faltante de compra", "excedente de malla", "holgura", "no es activo",
                  "Z/D_tot", "Z/ha", "plantas presentes",
                  "## No incluido / pendiente", "CP-SAT", "DSATUR", "κ", "convergencia",
                  "compra_d", "compra_nominal", "D_completo",
                  "π = 4 y π = 10", "mejor semilla", "γ > 100"]:
        assert frase in texto, frase


def test_brechas_paso3_valores():
    o = RAIZ / "results"
    b = resumen.brechas_paso3(o).set_index("instancia")
    assert len(b) == 31
    assert abs(b.loc["1ha", "huecos_por_ha"] - 12.2) < 0.1
    assert (b["huecos_por_ha"] > 4).all() and (b["huecos_por_ha"] < 13).all()
    assert np.allclose(b["faltante_compra"] + b["excedente_malla"], b["huecos"], atol=0.5)
