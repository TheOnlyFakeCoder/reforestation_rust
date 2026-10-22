import re

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
    assert re.search(r"\| total \| \d+ \| \d+ \| \d+ \| [\d.]+ \|", texto)
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
    assert "∞ (p3_a, basica)" in t  # la fila básica de p3_a tiene cota 0
    assert "CP-SAT" not in resumen._pendiente(tmp_path)
    # p3_a: la incumbente (34.9) mejora a la mejor semilla (35.07); p1_a y p3_b la igualan
    assert "mejora a la mejor semilla del tabú en 1 de 3 instancias" in t
    # (incumbente − cota)/cota de p3_a con la fila ajustada: (34.9 − 30)/30 = 16.3 %
    assert "entre 16.3 % y 16.3 %" in t
    assert "contra la incumbente de CP-SAT: p3_a: 0.0" in t
    assert "tolerancia" in t


def test_validacion_exacta_workers_por_formulacion(tmp_path):
    _validacion_sintetica(tmp_path)
    ruta = tmp_path / "validacion_exacta.csv"
    v = pd.read_csv(ruta)
    v.loc[v["formulacion"] == "ajustada", "workers"] = 32
    v.to_csv(ruta, index=False)
    t = resumen._validacion(tmp_path)
    assert "8 workers en la básica y 32 en la ajustada" in t


def test_validacion_exacta_sin_csv(tmp_path):
    t = resumen._validacion(tmp_path)
    assert "no se ha corrido" in t
    assert "CP-SAT" in resumen._pendiente(tmp_path)


# ---------------------------------------------------------------------------
# Secciones de la entrega final (fase 5), con archivos sintéticos en tmp_path.

import json
import shutil

J = 10
NIVELES = [f"Z_nivel{k}" for k in range(1, 6)] + ["Z_otro"]


def _entorno_sintetico(o):
    r = {"modelo_pc": "ACME Laptop X", "sistema_operativo": "Microsoft Windows 11 (versión 10.0)",
         "disco_total_gb": 953.2, "disco_libre_gb": 250.5, "tipo_disco": "SSD", "ram_gb": 31.7,
         "ram_modulos": [{"capacidad_gb": 16.0, "tipo": "DDR5", "velocidad_mhz": 5600},
                         {"capacidad_gb": 16.0, "tipo": "DDR5", "velocidad_mhz": 5600}],
         "procesador": "CPU Z", "nucleos_fisicos": 24, "hilos_logicos": 32,
         "commit": "abc123def", "software": {"rustc": "rustc 1.80.0", "python": "3.12.1",
                                             "pandas": "2.2.0"}}
    (o / "entorno.json").write_text(json.dumps(r), encoding="utf-8")
    pd.DataFrame([("reto datos", "", 1.5, "2026-10-21T10:00:00"),
                  ("reto paso3", "todas", 111.25, "2026-10-21T10:01:00"),
                  ("python python\\ajuste.py", "", 3.0, "2026-10-21T10:02:00"),
                  ("reto paso3", "todas", 222.75, "2026-10-21T11:00:00")],
                 columns=["paso", "instancias", "segundos", "fecha"]).to_csv(
        o / "tiempos.csv", index=False)


def _tamano_sintetico(o):
    pd.DataFrame([("1ha", "paso1_tabu", 658, "enteras", 30, "una por posición"),
                  ("1ha", "extensivo_completo", 123456789, "binarias y enteras", 98765432,
                   "10*|V| binarias; justifica la descomposición")],
                 columns=["instancia", "modelo", "variables", "tipo_variables", "restricciones",
                          "nota"]).to_csv(o / "tamano_modelo.csv", index=False)
    pd.DataFrame([("costo unitario", "c_j", 10, "data/especies.csv"),
                  ("matriz de interacción", "W", 100, "rasgos"),
                  ("TOTAL", "", 110, "suma")],
                 columns=["parametro", "simbolo", "cantidad", "fuente"]).to_csv(
        o / "tamano_parametros.csv", index=False)
    f = {"instancia": "1ha", "especies": 10, "plantas_por_ha": 658.0, "D_tot": 658, "n_pos": 658}
    f.update({f"D_{j}": 60 + j for j in range(J)})
    f.update({f"Dnom_{j}": 50 + j for j in range(J)})
    pd.DataFrame([f]).to_csv(o / "tamano_problema.csv", index=False)


def _experimentacion_sintetica(o):
    filas = []
    for inst in ("1ha", "map_27"):
        filas.append((inst, "T", None, 1000, 129.467, 3.36, 129.2586, 129.6754))
        for mag, base in (("X", 8.239), ("C", 37.761), ("n", 36.752), ("q", 38.0)):
            for j in range(J):
                m = base + j
                h = 0 if mag == "q" else 0.1765
                filas.append((inst, mag, j, 1000, m, 0 if mag == "q" else 2.8, m - h, m + h))
        filas.append((inst, "Z", None, 1000, 565.399, 11.08, 564.712, 566.086))
        for k, c in enumerate(NIVELES):
            filas.append((inst, c, None, 1000, 10.0 * (k + 1), 1.0, 9.9 * (k + 1), 10.1 * (k + 1)))
    t = pd.DataFrame(filas, columns=["instancia", "magnitud", "especie", "n", "media_ha", "sd_ha",
                                     "ic95_inf", "ic95_sup"])
    t["especie"] = t["especie"].astype("Int64")
    t.to_csv(o / "experimentacion.csv", index=False)


def _matriz(prob):
    return [(j, k, 5, 50, prob) for j in range(J) for k in range(J)]


def _transicion_sintetica(o):
    pd.DataFrame({"paso": range(1, 31), "pos": range(101, 131), "fila": 0, "col": range(30),
                  "x": [2.0 * i for i in range(30)], "y": 0.0, "especie": [i % 10 for i in range(30)]}
                 ).to_csv(o / "orden_plantacion_1ha.csv", index=False)
    filas = []
    for inst, patron, alcance, tipo, prob in [
            ("1ha", "tabu_paso3", "todos_los_escenarios", "orden", 0.0123456),
            ("1ha", "tabu_paso3", "todos_los_escenarios", "vecindad", 0.0456789),
            ("1ha", "tabu_paso3", "omega0", "orden", 0.0777),
            ("1ha", "tabu_paso3", "omega0", "vecindad", 0.0888),
            ("1ha", "practica_iii", "omega0", "orden", 0.0999),
            ("map_27", "tabu_paso3", "todos_los_escenarios", "orden", 0.0555)]:
        filas += [(inst, patron, alcance, tipo, *m) for m in _matriz(prob)]
    pd.DataFrame(filas, columns=["instancia", "patron", "alcance", "tipo", "de", "a", "conteo", "n",
                                 "prob"]).to_csv(o / "transicion.csv", index=False)
    pd.DataFrame([("1ha", "tabu_paso3", "todos_los_escenarios", j, 0.1111, 0.1222) for j in range(J)]
                 + [("1ha", "tabu_paso3", "omega0", j, 0.3333, 0.1222) for j in range(J)],
                 columns=["instancia", "patron", "alcance", "especie", "pi_estacionaria",
                          "composicion_plantada"]).to_csv(o / "transicion_estacionaria.csv",
                                                          index=False)


def _sensibilidad_sintetica(o):
    filas = []
    for var, inst, z in [("base", "1ha", 500.0), ("kappa_0.4", "1ha", 450.0),
                         ("base", "map_21", 900.0), ("kappa_0.4", "map_21", 800.0)]:
        f = {"variante": var, "instancia": inst}
        f.update({f"D_{j}": 60 + j for j in range(J)})
        f.update(suma_theta_d=12.5, z_opt_media=z, z_opt_sd=1.0, costo_compra=12000.0,
                 faltante_esperado=10.0, z_paso3_media=np.nan if inst == "map_21" else z + 60,
                 z_paso3_sd=np.nan, z_practica_iii=np.nan, z_dsatur_con=np.nan, z_tabu_con=np.nan,
                 cambio_z_opt_pct=0.0 if var == "base" else -10.0,
                 cambio_z_paso3_pct=np.nan)
        filas.append(f)
    pd.DataFrame(filas).to_csv(o / "sensibilidad.csv", index=False)


def _practica_con_dsatur(o):
    for f in ("practica_compra.csv", "markov.csv", "malla_resumen.csv"):
        shutil.copy(RAIZ / "results" / f, o / f)
    d = pd.read_csv(RAIZ / "results" / "practica_distribucion.csv")
    # no depende de si results/ ya trae filas DSATUR: se descartan y se agregan las sintéticas
    d = d[~d["variante"].astype(str).str.startswith("dsatur")]
    extra = []
    for inst in ("1ha", "map_27"):
        for var, z in (("dsatur_sin_existentes", 780.0), ("dsatur_con_existentes", 700.0)):
            f = d[(d["instancia"] == inst) & (d["variante"] == "tabu_con_existentes")].iloc[0].copy()
            f["variante"], f["Z"] = var, z
            extra.append(f)
    pd.concat([d, pd.DataFrame(extra)], ignore_index=True).to_csv(
        o / "practica_distribucion.csv", index=False)


def test_entorno_y_tiempos(tmp_path):
    _entorno_sintetico(tmp_path)
    t = resumen._entorno(tmp_path)
    for frase in ["ACME Laptop X", "Windows 11", "CPU Z", "24 núcleos físicos", "32 hilos",
                  "DDR5", "5600 MHz", "16 GB", "953.2 GB", "SSD", "abc123def", "rustc 1.80.0",
                  "| pandas | 2.2.0 |"]:
        assert frase in t, frase
    # tiempos: la última corrida de cada paso
    assert "| reto paso3 | todas | 222.75 |" in t
    assert "111.25" not in t
    assert "| reto datos |  | 1.5 |" in t


def test_entorno_sin_archivos(tmp_path):
    t = resumen._entorno(tmp_path)
    assert "Falta `entorno.json`" in t and "Falta `tiempos.csv`" in t


def test_tamano_problema(tmp_path):
    _tamano_sintetico(tmp_path)
    t = resumen._tamano(tmp_path)
    assert "| TOTAL |  | 110 |" in t
    assert "extensivo_completo" in t and "123456789" in t and "forma extensiva" in t
    assert "Dnom_9" in t and "658" in t
    assert r"10*\|V\| binarias" in t  # las barras dentro de una celda no rompen la tabla
    vacio = resumen._tamano(tmp_path / "no")
    for f in ("tamano_modelo.csv", "tamano_parametros.csv", "tamano_problema.csv"):
        assert f"Falta `{f}`" in vacio


def test_experimentacion_seccion(tmp_path):
    _experimentacion_sintetica(tmp_path)
    t = resumen._experimentacion(tmp_path)
    for frase in ["E[X_j]/ha", "E[C_j]/ha", "q_j/ha", "E[n_j]/ha", "E[T]/ha", "E[Z]/ha",
                  "IC 95 %", "1.96", "Z_nivel1", "Z_otro", "### 1ha", "### map_27"]:
        assert frase in t, frase
    assert "129.47" in t and "[129.26, 129.68]" in t
    assert "8.24" in t and "[8.06, 8.42]" in t
    assert "565.4" in t and "[564.71, 566.09]" in t
    assert "29 instancias" in t and "100 escenarios" in t
    assert "Falta `experimentacion.csv`" in resumen._experimentacion(tmp_path / "no")


def test_orden_y_transiciones(tmp_path):
    _transicion_sintetica(tmp_path)
    t = resumen._orden_transiciones(tmp_path)
    assert "| 21 | 121 |" in t and "| 22 | 122 |" not in t
    principal, ejemplo = t.split("ω = 0 (ejemplo")
    assert "0.012" in principal and "0.046" in principal  # orden y vecindad agregadas, 3 decimales
    assert "0.0123" not in t
    assert "0.078" in ejemplo and "0.089" in ejemplo
    assert "0.078" not in principal
    assert "0.100" not in t  # la de la práctica (iii) no se muestra
    assert "0.055" not in t  # map_27 no se muestra
    assert "| 0 | 0.111 | 0.122 |" in t  # estacionaria contra composición
    assert "Falta `orden_plantacion_1ha.csv`" in resumen._orden_transiciones(tmp_path / "no")


def test_sensibilidades_seccion(tmp_path):
    _sensibilidad_sintetica(tmp_path)
    t = resumen._sensibilidades(tmp_path)
    assert "kappa_0.4" in t and "map_21" in t and "κ" in t and "-10" in t
    assert "Falta `sensibilidad.csv`" in resumen._sensibilidades(tmp_path / "no")


def test_practica_con_dsatur_y_pendiente(tmp_path):
    assert "DSATUR" in resumen._pendiente(tmp_path)
    assert "Sensibilidades" in resumen._pendiente(tmp_path)
    _practica_con_dsatur(tmp_path)
    _sensibilidad_sintetica(tmp_path)
    t = resumen._practica(tmp_path)
    assert "dsatur_con_existentes" in t
    frase = [ln for ln in t.splitlines() if ln.startswith("DSATUR por cuotas")]
    assert len(frase) == 1
    assert "700" in frase[0] and "tabú" in frase[0] and "serpentina" in frase[0]
    p = resumen._pendiente(tmp_path)
    assert "DSATUR" not in p and "Sensibilidades" not in p


def test_figuras_nuevas():
    f = resumen._figuras()
    for n in ("orden_plantacion_1ha.png", "orden_plantacion_map_27.png", "sensibilidad.png"):
        assert f"../figures/{n}" in f


def test_resumen_secciones_entrega_final(tmp_path):
    texto = resumen.main(origen=RAIZ / "results", salida=tmp_path / "R.md").read_text(encoding="utf-8")
    titulos = ["## Entorno y software", "## Tamaño del problema", "## Malla",
               "## Práctica actual", "## Experimentación (1000 simulaciones)",
               "## Orden de plantación y transiciones", "## Sensibilidades",
               "## Convergencia del tabú"]
    pos = [texto.index(x) for x in titulos]
    assert pos == sorted(pos)


def test_frase_dsatur_tolera_filas_duplicadas():
    filas = []
    for var, z in (("tabu_con_existentes", 500.0), ("dsatur_con_existentes", 700.0),
                   ("dsatur_con_existentes", 710.0), ("dsatur_sin_existentes", 780.0)):
        filas.append({"instancia": "1ha", "variante": var, "Z": z})
    t = resumen._frase_dsatur(pd.DataFrame(filas))
    assert "710.0" in t and "700.0" not in t  # se queda con la última fila
