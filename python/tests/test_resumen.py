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


def test_resumen_convergencia_y_calibracion(tmp_path):
    texto = resumen.main(origen=RAIZ / "results", salida=tmp_path / "R.md").read_text(encoding="utf-8")
    for titulo in ["## Convergencia del tabú", "### Motivos de paro", "### Corrida de referencia",
                   "### Calibración de K y tenure", "### Parámetros finales del tabú"]:
        assert titulo in texto, titulo
    assert "| total | 3382 | 0 | 3382 | 1 |" in texto
    assert "0.604" in texto and "iter_por_posicion" in texto and "300" in texto
    assert "z_opt_sd" in texto
    assert "una semilla por escenario" in texto
    assert "convergencia_map_21.png" in texto
    assert "Curvas de convergencia de un polígono de 8 ha" not in texto
    assert "calibración" not in texto.split("## No incluido / pendiente")[1].split("## Figuras")[0].lower()
    assert "CP-SAT" in texto


def test_calibracion_eleccion_regla():
    c = pd.read_csv(RAIZ / "results" / "calibracion_tabu.csv")
    k, t = resumen.eleccion_calibracion(c)
    assert (k, t) == (128, 10)


def _validacion_sintetica(o):
    import math
    import validacion_exacta as ve
    base = {c: 0 for c in ve.COLUMNAS}
    filas = []
    for id_, mod, form, opt, z, cota, ts, zts, est, libres in [
        ("p1_a", "paso1", "basica", True, 200.0, 200.0, 200.0, 201.0, "OPTIMAL", 1),
        ("p3_a", "paso3", "basica", False, 35.0, 0.0, 35.07, 35.2, "FEASIBLE", None),
        ("p3_a", "paso3", "ajustada", False, 34.9, 30.0, 35.07, 35.2, "FEASIBLE", None),
        ("p3_b", "paso3", "basica", True, 40.0, 40.0, 40.0, 40.4, "OPTIMAL", None),
    ]:
        ref = z if opt else cota
        f = dict(base, id=id_, modelo=mod, formulacion=form, hint=True, n_pos=49,
                 n_especies_libres=libres, z_opt=z, cota=cota, optimo_probado=opt,
                 tiempo_cpsat_s=12.5, z_ts_mejor=ts, z_ts_media=zts, z_ts_sd=0.1,
                 brecha_mejor=ve.brecha(ts, ref), brecha_media=ve.brecha(zts, ref),
                 frac_semillas_optimo=0.3 if opt else 0.0, z_comp_opt=60.0, z_comp_ts_mejor=60.3,
                 brecha_z=0.005 if mod == "paso1" else ve.brecha(ts, ref),
                 error_redondeo_max=4.9e-5, estado=est, tolerancia_redondeo=0.01,
                 brecha_vs_incumbente=ve.brecha(ts, z), tiempo_limite_s=600.0, workers=8)
        filas.append(f)
    ve.escribir_csv(filas, o / "validacion_exacta.csv")
    assert math.isinf(filas[1]["brecha_mejor"])


def test_validacion_exacta_seccion(tmp_path):
    _validacion_sintetica(tmp_path)
    t = resumen._validacion(tmp_path)
    assert "| p3_a | paso3 | ajustada |" in t
    assert "600 s" in t and "8 workers" in t and "hint" in t
    assert "2 de 3 instancias" in t
    assert "p3_a" in t.split("Sin óptimo probado")[1]
    # brecha definitiva de p3_a: ajustada (cota 30): (35.07 − 30)/30 = 16.9 %
    assert "16.9" in t
    assert "brecha_z" in t and "0.5 %" in t
    assert "p1_a: 1" in t
    assert "∞" in t  # la fila básica de p3_a tiene cota 0
    assert "CP-SAT" not in resumen._pendiente(tmp_path)


def test_validacion_exacta_sin_csv(tmp_path):
    t = resumen._validacion(tmp_path)
    assert "no se ha corrido" in t
    assert "CP-SAT" in resumen._pendiente(tmp_path)
