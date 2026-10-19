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
