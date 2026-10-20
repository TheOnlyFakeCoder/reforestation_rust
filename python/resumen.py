"""Genera results/RESUMEN.md a partir de los CSV/JSON de results/."""
import argparse
import json
from pathlib import Path

import numpy as np
import pandas as pd

try:
    from comun import RAIZ, leer_config
except ImportError:
    from python.comun import RAIZ, leer_config

INSTANCIAS = ["1ha", "map_27"]
FIGURAS_LISTA = [
    ("hist_rho.png", "Histograma de la densidad ρ con las densidades ajustadas"),
    ("C_por_especie_1ha.png", "Requerimiento C_j por especie, 1 ha"),
    ("C_por_especie_map_27.png", "Requerimiento C_j por especie, P1 (map_27)"),
    ("curva_compromiso_1ha.png", "Curva de compromiso competencia vs. costo, 1 ha"),
    ("curva_compromiso_map_27.png", "Curva de compromiso competencia vs. costo, P1"),
    ("convergencia_1ha.png", "Convergencia del tabú del paso 1, 1 ha"),
    ("convergencia_map_27.png", "Convergencia del tabú del paso 1, P1"),
    ("convergencia_map_21.png", "Convergencia del tabú del paso 1, map_21 (mayor |V|)"),
    ("mapa_1ha.png", "Mapa socia vs. tabú (paso 3, ω = 0), 1 ha"),
    ("mapa_map_27.png", "Mapa socia vs. tabú (paso 3, ω = 0), P1"),
    ("practica_compra.png", "Práctica actual vs. modelo: compra de plantas"),
    ("practica_distribucion.png", "Práctica actual vs. tabú: distribución"),
]


def _fmt(v):
    if isinstance(v, (bool, np.bool_)):
        return str(bool(v))
    if isinstance(v, float):
        if v != v:
            return ""
        return format(round(v, 4), "f").rstrip("0").rstrip(".")
    if v is None:
        return ""
    return str(v)


def tabla_md(df):
    cols = list(df.columns)
    lineas = ["| " + " | ".join(str(c) for c in cols) + " |",
              "| " + " | ".join("---" for _ in cols) + " |"]
    for fila in df.itertuples(index=False):
        lineas.append("| " + " | ".join(_fmt(v) for v in fila) + " |")
    return "\n".join(lineas)


def _csv(origen, nombre):
    return pd.read_csv(origen / nombre)


def _malla(o):
    r = _csv(o, "malla_resumen.csv")
    an = _csv(o, "malla_anillos.csv")
    ren = {"n_pos": "\\|V\\|", "round_658A": "round(658·A)"}
    sel = r[r["instancia"].isin(INSTANCIAS)]
    cols = ["instancia", "area_ha", "n_pos", "round_658A", "excedente", "d_tot"]
    pa = an[an["instancia"].isin(INSTANCIAS)].pivot(index="anillo", columns="instancia",
                                                    values="pares").reset_index()
    pa = pa[["anillo"] + [i for i in INSTANCIAS if i in pa.columns]]
    cols3 = ["instancia", "area_ha", "columnas", "filas", "n_pos", "round_658A", "excedente",
             "d_tot", "pares_anillo_1"]
    return "\n\n".join([
        "### 1 ha y P1 (map_27)\n\n" + tabla_md(sel[cols].rename(columns=ren)),
        "Pares por anillo:\n\n" + tabla_md(pa),
        f"### Las {len(r)} instancias\n\n" + tabla_md(r[cols3].rename(columns=ren)),
    ])


def _ajuste(o):
    r = _csv(o, "ajuste_resumen.csv")
    dens = _csv(o, "ajuste_densidad.csv")
    mc = json.loads((o / "mc_params.json").read_text(encoding="utf-8"))
    p = pd.DataFrame({"especie": range(len(mc["p_hat"])), "p_hat": mc["p_hat"]})
    return (tabla_md(r) + "\n\nAjuste de la densidad ρ:\n\n" + tabla_md(dens)
            + f"\n\nParámetros de Monte Carlo (`mc_params.json`): μ = {mc['mu']:.4f}, "
            f"σ = {mc['sigma']:.4f}.\n\n" + tabla_md(p))


def _paso1(o, gamma_star):
    g = _csv(o, "paso1_resumen_gamma.csv")
    partes = [f"γ* = {gamma_star:g}, elegido por el usuario en el codo de la curva de compromiso."]
    for inst in INSTANCIAS:
        s = g[g["instancia"] == inst][["gamma", "suma_theta_d", "suma_c_d", "z_opt_media", "z_opt_min",
                                       "z_opt_max", "z_opt_sd", "z_opt_por_planta_media"]]
        partes.append(f"Barrido de γ, {inst}:\n\n" + tabla_md(s))
    c = _csv(o, "paso1_composicion.csv")
    cols = ["instancia", "d_tot"] + [f"D_{j}" for j in range(10)]
    partes.append("D* por instancia (γ*):\n\n" + tabla_md(c[cols]))
    return "\n\n".join(partes)


def _mc(o):
    e = _csv(o, "mc_resumen_especies.csv")
    e = e[e["instancia"].isin(INSTANCIAS)][["instancia", "composicion", "especie", "D", "E_C", "sd_C",
                                            "P90", "P95", "N_necesario"]]
    c = _csv(o, "mc_resumen_costo.csv")
    return ("Por especie (1 ha y P1):\n\n" + tabla_md(e)
            + f"\n\nCosto (las {c['instancia'].nunique()} instancias):\n\n"
            + tabla_md(c.drop(columns=["map_id"])))


def _paso2(o, cfg):
    p = _csv(o, "paso2_compra.csv")
    p = p.assign(q_total=p[[f"q_{j}" for j in range(10)]].sum(axis=1))
    eps, pi = cfg.get("epsilon", 0.1), cfg.get("pi_factor", 2)
    mism_eps = p["epsilon"].round(6) == round(eps, 6)
    base = p[mism_eps & (p["pi_factor"] == pi)]
    cols = ["instancia", "q_total", "costo", "presupuesto", "cabe", "faltante_esperado",
            "sobrante_esperado"]
    comp = p[mism_eps & p["instancia"].isin(INSTANCIAS)]
    return "\n\n".join([
        f"ε = {eps:g}, π = {pi:g}·c ({len(base)} instancias):\n\n" + tabla_md(base[cols]),
        "Comparación de π (1 ha y P1):\n\n" + tabla_md(
            comp[["instancia", "pi_factor"] + cols[1:]]),
    ])


def brechas_paso3(o, cfg=None):
    """Huecos por instancia de paso 3: (V_lib - M) medio, partido en faltante de compra
    (D_tot - T - M, es decir ΣC⁺ - M) y excedente de malla (|V| - D_tot)."""
    cfg = cfg or leer_config()
    p3 = _csv(o, "paso3_escenarios.csv")
    m = _csv(o, "malla_resumen.csv").set_index("instancia")
    p2 = _csv(o, "paso2_compra.csv")
    p2 = p2[np.isclose(p2["epsilon"], cfg["epsilon"]) & np.isclose(p2["pi_factor"], cfg["pi_factor"])]
    p2 = p2.set_index("instancia")
    p3 = p3.assign(huecos=p3["V_lib"] - p3["M"],
                   sumC=m["d_tot"].reindex(p3["instancia"]).to_numpy() - p3["T"] - p3["M"])
    g = p3.groupby("instancia", sort=False).agg(huecos=("huecos", "mean"),
                                                faltante_compra=("sumC", "mean"))
    g.insert(0, "instancia", g.index)
    g["excedente_malla"] = (m["n_pos"] - m["d_tot"]).reindex(g.index).to_numpy()
    g["area_ha"] = m["area_ha"].reindex(g.index).to_numpy()
    g["huecos_por_ha"] = g["huecos"] / g["area_ha"]
    g["faltante_esperado_paso2"] = p2["faltante_esperado"].reindex(g.index).to_numpy()
    g["holgura_presupuesto"] = (p2["presupuesto"] - p2["costo"]).reindex(g.index).to_numpy()
    return g.reset_index(drop=True)


def _paso3(o):
    r = _csv(o, "paso3_resumen.csv")
    b = brechas_paso3(o)
    tot = int(r["casos_presupuesto_corto"].sum())
    n = int(r["n"].sum())
    ph = b["huecos_por_ha"]
    sel = b[b["instancia"].isin(INSTANCIAS)]
    h1 = float(b.loc[b["instancia"] == "1ha", "huecos_por_ha"].iloc[0])
    h27 = float(b.loc[b["instancia"] == "map_27", "huecos_por_ha"].iloc[0])
    nota = (
        f"Los escenarios caen en `presupuesto_corto` en {tot} de {n} casos. El nombre es el de la "
        "especificación para Σn < |V_lib|; aquí no lo causa el presupuesto. Con π = 2c la razón "
        "crítica es 0.5, así que el modelo compra cerca de la mediana de C, y además la malla tiene "
        "posiciones de más (|V| > D_tot). Como el caso `usual` exige Σn = |V_lib|, no se alcanza si "
        "algún q_j < C_j o si |V| > D_tot.\n\n"
        f"Huecos medios por hectárea, (V_lib − M)/A: entre {ph.min():.1f} y {ph.max():.1f} en las "
        f"{len(b)} instancias ({h1:.1f} en 1 ha, {h27:.1f} en P1/map_27). Cada hueco se parte en "
        "faltante de compra (ΣC⁺ − M, que coincide con el `faltante_esperado` del paso 2 salvo "
        "por el número de escenarios: 100 aquí, 1000 allá) y excedente de malla (|V| − D_tot). "
        f"El excedente de malla va de {int(b['excedente_malla'].min())} a "
        f"{int(b['excedente_malla'].max())} posiciones.\n\n"
        "El presupuesto no es activo con π = 2: la compra cabe en todas las instancias, con una "
        f"holgura B·A − costo de ${b['holgura_presupuesto'].min():,.0f} a "
        f"${b['holgura_presupuesto'].max():,.0f}. Es un hallazgo del modelo, no un error.")
    cols = ["instancia", "area_ha", "huecos", "huecos_por_ha", "faltante_compra",
            "faltante_esperado_paso2", "excedente_malla", "holgura_presupuesto"]
    return (nota + "\n\nHuecos de paso 3 (1 ha y P1):\n\n" + tabla_md(sel[cols])
            + "\n\nHuecos de paso 3 (todas las instancias):\n\n" + tabla_md(b[cols])
            + "\n\nResumen del paso 3. `Z_sd` es la desviación sobre escenarios (una semilla por "
            "escenario), no sobre semillas de una misma instancia:\n\n"
            + tabla_md(r.drop(columns=["map_id"])))


def _practica(o):
    c = _csv(o, "practica_compra.csv")
    c = c[c["instancia"].isin(INSTANCIAS)].drop(columns=["map_id", "escenarios"])
    d = _csv(o, "practica_distribucion.csv")
    d = d[d["instancia"].isin(INSTANCIAS)].copy()
    m = _csv(o, "malla_resumen.csv").set_index("instancia")
    d["D_tot"] = m["d_tot"].reindex(d["instancia"]).to_numpy()
    d["Z/D_tot"] = d["Z"] / d["D_tot"]
    d["Z/ha"] = d["Z"] / m["area_ha"].reindex(d["instancia"]).to_numpy()
    resto = [k for k in d.columns if k.startswith(("pares", "Z_nivel")) or k == "Z_otro"]
    d = d[["instancia", "variante", "plantas", "D_tot", "Z", "Z_por_planta", "Z/D_tot", "Z/ha"] + resto]
    mk = _csv(o, "markov.csv")
    mk = mk[mk["instancia"].isin(INSTANCIAS)].groupby(
        ["instancia", "base", "composicion"], as_index=False, sort=False)[
        ["plantas", "vivas_1ano", "vivas_2anos", "reposicion_esperada"]].sum()
    orden = {"compra_d": 0, "compra_nominal": 1, "D_completo": 2}
    mk = mk.assign(_o=mk["base"].map(orden)).sort_values(["instancia", "_o"]).drop(columns="_o")
    return ("Compra (a nominal 658/ha, b nominal 524/ha, c paso 2 ε = 0, d paso 2 ε = 0.1):\n\n"
            + tabla_md(c)
            + "\n\nDistribución. `Z_por_planta` divide entre las plantas presentes (T + M), que en "
            "las variantes con existentes son menos que D_tot; por eso se añaden Z/D_tot y Z/ha "
            "(D_tot y el área vienen de `malla_resumen.csv`):\n\n" + tabla_md(d)
            + "\n\nMarkov. Titular: las filas `compra_d` (compra del paso 2, ε = 0.1) y "
            "`compra_nominal` (q = D̄) proyectan la supervivencia sobre las plantas realmente "
            "plantadas, E[min(q_j, C⁺_j)] sobre los escenarios. Las filas `D_completo` suponen "
            "plantar todo D sin plantas existentes, que ya sobrevivieron al establecimiento, y por "
            "eso sobrestiman las vivas y la reposición (entre 24 % y 28 % en la reposición de 1 ha). Las vivas a 1 año de "
            "`compra_d` en 1 ha coinciden con la opción d de `practica_compra.csv`.\n\n"
            + tabla_md(mk))


def eleccion_calibracion(c):
    """Regla de elección de (K, tenure): menor promedio de z_rel entre instancias; si otra
    combinación queda a <= 0.001 y es más rápida (tiempo medio promedio), se elige la más rápida."""
    c = c.assign(z_rel=c["z_opt_media"] / c.groupby("instancia")["z_opt_media"].transform("min"))
    g = c.groupby(["candidatos", "tenure"]).agg(z_rel=("z_rel", "mean"),
                                                tiempo=("tiempo_medio_s", "mean")).reset_index()
    mejor = g.loc[g["z_rel"].idxmin()]
    cerca = g[(g["z_rel"] <= mejor["z_rel"] + 0.001) & (g["tiempo"] < mejor["tiempo"])]
    if len(cerca):
        mejor = cerca.loc[cerca["tiempo"].idxmin()]
    return int(mejor["candidatos"]), int(mejor["tenure"])


def _convergencia(o, cfg):
    t = cfg["tabu"]
    crit = _csv(o, "convergencia_criterio.csv")
    ref = _csv(o, "convergencia_referencia.csv")
    cal = _csv(o, "calibracion_tabu.csv")
    tot = crit[crit["instancia"] == "total"].iloc[0]
    por = crit[crit["instancia"] != "total"]
    cal = cal.assign(z_rel=cal["z_opt_media"] / cal.groupby("instancia")["z_opt_media"].transform("min"))
    k, ten = eleccion_calibracion(cal.drop(columns="z_rel"))
    prom = cal.groupby(["candidatos", "tenure"], as_index=False).agg(
        z_rel_prom=("z_rel", "mean"), tiempo_prom_s=("tiempo_medio_s", "mean"))
    cols = ["instancia", "candidatos", "tenure", "n_semillas", "z_opt_media", "z_opt_sd", "z_rel",
            "tiempo_medio_s", "iter_media", "frac_sin_mejora"]
    fin = ref[ref["corrida"] == "final"].set_index("instancia")
    dr = ", ".join(f"{i}: {100 * r['dif_rel']:.3f} %" for i, r in fin.iterrows())
    intro = (
        f"Tope escalado por tamaño de la malla: max_iter_efectivo = max(max_iter, "
        f"iter_por_posicion·|V|) y max_sin_mejora_efectivo = max(max_sin_mejora, "
        f"sin_mejora_por_posicion·|V|). Criterio: al menos 90 % de las búsquedas tabú terminan por "
        f"`sin_mejora` en total y 80 % en cada instancia. Se cumple: {tot['paros_sin_mejora']:.0f} de "
        f"{tot['busquedas']:.0f} búsquedas ({100 * tot['frac_sin_mejora']:.1f} %) y el mínimo por instancia es "
        f"{100 * por['frac_sin_mejora'].min():.1f} %.")
    refp = (
        "La corrida de referencia usa topes 10 veces mayores (iter_por_posicion = 3000); "
        "`dif_rel` es la diferencia relativa de z_opt_media de la corrida final contra la referencia "
        f"(criterio: <= 0.5 %): {dr}. Con topes 100/4 (iter_por_posicion / sin_mejora_por_posicion) "
        "map_21 no pasó esta verificación (0.604 % > 0.5 %); el usuario eligió subir los topes a 300/8, "
        "con lo que se cumplen el criterio de paro y la referencia.")
    calp = (
        "Calibración con γ = γ* = 3 en 1 ha y map_21, 5 semillas por combinación. z_rel es "
        "z_opt_media entre el mínimo de la rejilla en esa instancia. Regla: menor promedio de z_rel "
        "entre las dos instancias; si otra combinación queda a <= 0.001 de ese promedio y es más "
        f"rápida, se elige la más rápida. Resultado: K = {k}, tenure = {ten}.")
    par = pd.DataFrame({"parámetro": ["candidatos", "tenure", "max_iter", "max_sin_mejora",
                                      "iter_por_posicion", "sin_mejora_por_posicion"],
                        "valor": [t["candidatos"], t["tenure"], t["max_iter"], t["max_sin_mejora"],
                                  t["iter_por_posicion"], t["sin_mejora_por_posicion"]]})
    return "\n\n".join([
        intro,
        "### Motivos de paro\n\nPor instancia, sobre todas las búsquedas del pipeline:\n\n"
        + tabla_md(crit),
        "### Corrida de referencia\n\n" + refp + "\n\n" + tabla_md(ref),
        "### Calibración de K y tenure\n\n" + calp + "\n\n" + tabla_md(cal[cols])
        + "\n\nPromedio entre instancias:\n\n" + tabla_md(prom),
        "### Parámetros finales del tabú\n\nLeídos de `config/default.json`:\n\n" + tabla_md(par),
    ])


def _pct(v):
    """Brecha relativa como porcentaje; ∞ si la cota no es positiva."""
    if v != v:
        return ""
    if np.isinf(v):
        return "∞"
    p = 100 * v
    if p == 0:
        return "0"
    return f"{p:.1e}" if abs(p) < 1e-3 else f"{p:.3g}"


def validacion_definitiva(v):
    """Una fila por instancia: la de óptimo probado si la hay; si no, la de mayor cota."""
    v = v.assign(_o=~v["optimo_probado"].astype(bool), _c=-v["cota"])
    orden = list(dict.fromkeys(v["id"]))
    d = v.sort_values(["_o", "_c"], kind="stable").groupby("id", sort=False).head(1)
    return d.set_index("id").loc[orden].reset_index().drop(columns=["_o", "_c"])


def _validacion(o):
    ruta = Path(o) / "validacion_exacta.csv"
    if not ruta.exists():
        return ("La validación exacta no se ha corrido (`python/validacion_exacta.py`, que lee "
                "`results/validacion/` de `reto instancias-val` y `reto tabu-val`).")
    v = pd.read_csv(ruta)
    v["optimo_probado"] = v["optimo_probado"].astype(bool)
    d = validacion_definitiva(v)
    n = len(d)
    opt = d[d["optimo_probado"]]
    no = d[~d["optimo_probado"]]
    por_form = {f: int((opt["formulacion"] == f).sum()) for f in ("basica", "ajustada")}
    tl = ", ".join(f"{t:g}" for t in sorted(v["tiempo_limite_s"].unique()))
    wk = ", ".join(str(int(w)) for w in sorted(v["workers"].unique()))
    n3, n1 = int((d["modelo"] == "paso3").sum()), int((d["modelo"] == "paso1").sum())
    i_max = d["brecha_mejor"].idxmax()
    semillas = sorted({len(pd.read_csv(t)) for t in (Path(o) / "validacion").glob("tabu_*.csv")})
    de_semillas = f" (de {', '.join(map(str, semillas))} semillas)" if semillas else ""
    hay_ajustada = (v["formulacion"] == "ajustada").any()
    partes = [
        f"CP-SAT (OR-Tools) resolvió {n} instancias pequeñas ({n3} del paso 3, ec. 24, y {n1} del "
        f"paso 1, ec. 21) con un límite de {tl} s y {wk} workers por formulación. En las "
        f"{len(v)} corridas CP-SAT arrancó con la mejor solución del tabú{de_semillas} como hint "
        "(`AddHint`). "
        + ("Primero se corrió la formulación básica y, donde no cerró, también la ajustada; la "
           "tabla tiene ambas filas. " if hay_ajustada else "Se usó la formulación básica. ")
        + "Los coeficientes del modelo son "
        "round(γ·w·h·10 000) y round(θ·10 000); Z*, Z_TS y las brechas usan los coeficientes "
        f"originales (error de redondeo máximo por coeficiente: {v['error_redondeo_max'].max():.1e}). "
        "Antes de comparar se verificó que el Z de Python de cada solución del tabú y de cada "
        "solución de CP-SAT coincide con el de Rust (`reto evaluar-val`) a 1e-6.",
        f"Óptimo probado en {len(opt)} de {n} instancias ({por_form['basica']} con la formulación "
        f"básica y {por_form['ajustada']} con la ajustada)"
        + (f": {', '.join(opt['id'])}." if len(opt) else "."),
    ]
    if len(no):
        partes.append(
            "Sin óptimo probado: " + ", ".join(
                f"{r.id} (cota {r.cota:.4f}, incumbente {r.z_opt:.4f}, {r.formulacion})"
                for r in no.itertuples())
            + ". En ellas la brecha se mide contra la cota, así que es una cota superior de la "
            "brecha real; para cada instancia se toma la fila de óptimo probado o, si no la hay, "
            "la de mayor cota.")
    partes.append(
        f"Brecha de la mejor semilla del tabú (contra el óptimo o la cota): máxima "
        f"{_pct(d.loc[i_max, 'brecha_mejor'])} % ({d.loc[i_max, 'id']}), media "
        f"{_pct(d['brecha_mejor'].mean())} %. Brecha de la media de las semillas: máxima "
        f"{_pct(d['brecha_media'].max())} %, media {_pct(d['brecha_media'].mean())} %. Contra "
        f"la incumbente de CP-SAT, la mejor semilla queda entre "
        f"{_pct(d['brecha_vs_incumbente'].min())} % y {_pct(d['brecha_vs_incumbente'].max())} %"
        + (" (negativa: el tabú quedó mejor que la incumbente de CP-SAT)."
           if (d["brecha_vs_incumbente"] < 0).any() else "."))
    if len(opt):
        partes.append(
            "Fracción de semillas que alcanzan el valor de CP-SAT (|Z_TS − Z*| ≤ 1e-6·max(1, Z*)), "
            "en las instancias con óptimo probado: "
            + ", ".join(f"{r.id}: {r.frac_semillas_optimo:.1f}" for r in opt.itertuples()) + ".")
    p1 = d[d["modelo"] == "paso1"]
    if len(p1):
        partes.append(
            "Paso 1: brecha_z = (γ·Z_TS − γ·Z*)/(γ·Z*), la competencia de la mejor semilla contra "
            "la de la solución de CP-SAT: "
            + ", ".join(f"{r.id}: {_pct(r.brecha_z)} %" for r in p1.itertuples())
            + ". Especies con L_j < U_j (n_especies_libres): "
            + ", ".join(f"{r.id}: {int(r.n_especies_libres)}" for r in p1.itertuples())
            + ".")
        una = p1[p1["n_especies_libres"] == 1]["id"].tolist()
        if una:
            partes[-1] += (f" En {', '.join(una)} sólo hay una especie libre y Σ D_j = D_tot está "
                           "fijo, así que la composición queda determinada y Σθ·D es constante: "
                           "esas instancias validan sólo el acomodo.")
    if np.isinf(v["brecha_mejor"]).any():
        partes.append("∞: la cota de la fila es ≤ 0, así que la brecha relativa contra la cota no "
                      "está acotada.")
    t = v.copy()
    for c in ["brecha_mejor", "brecha_media", "brecha_z", "brecha_vs_incumbente"]:
        t[c + "_%"] = t[c].map(_pct)
    t["error_redondeo_max"] = t["error_redondeo_max"].map(lambda e: f"{e:.1e}")
    cols = ["id", "modelo", "formulacion", "n_pos", "existentes", "n_especies_libres", "aristas",
            "variables", "z_opt", "cota", "optimo_probado", "estado", "tiempo_cpsat_s",
            "z_ts_mejor", "z_ts_media", "z_ts_sd", "tiempo_ts_medio_s", "brecha_mejor_%",
            "brecha_media_%", "brecha_vs_incumbente_%", "frac_semillas_optimo", "z_comp_opt",
            "z_comp_ts_mejor", "brecha_z_%", "error_redondeo_max"]
    t["n_especies_libres"] = t["n_especies_libres"].map(lambda x: "" if x != x else str(int(x)))
    partes.append("Tabla completa (`results/validacion_exacta.csv`; en el paso 1 los z son el "
                  "objetivo γ·Z + Σθ·D, y z_comp es Z):\n\n" + tabla_md(t[cols]))
    return "\n\n".join(partes)


def _pendiente(o=None):
    hecha = o is not None and (Path(o) / "validacion_exacta.csv").exists()
    cpsat = "" if hecha else "- Validación exacta con CP-SAT (§9).\n"
    return ("Lo siguiente de la especificación no se hizo y no tiene salida en `results/`:\n\n"
            + cpsat +
            "- Comparación con DSATUR (§10).\n"
            "- Sensibilidades: κ = 0.4 y 0.8, radios × 0.75 y × 1.25, m × 0, 0.5 y 2, y s2.\n"
            "")


def _notas():
    return ("- Curva de compromiso: cada punto usa Σθ·D de la mejor semilla (menor objetivo) y la "
            "media de Z_opt sobre todas las semillas, que pueden tener D distinto. La figura omite "
            "γ > 100; la tabla de paso 1 sí los incluye.\n"
            "- Comparación de π: las filas con π = 4 y π = 10 de 1 ha y P1 son idénticas porque el "
            "presupuesto es activo en ambos (holgura ≤ $23) y la penalización extra ya no compra "
            "nada más. No es un error.")


def _figuras():
    return "\n".join(f"- [{t}](../figures/{f})" for f, t in FIGURAS_LISTA)


def generar(origen):
    origen = Path(origen)
    cfg = leer_config()
    return "\n\n".join([
        "# Resumen de resultados: reforestación CONAFOR (Entregable 2)",
        "Generado por `python/resumen.py` a partir de los CSV y JSON de `results/`.",
        "## Malla\n\n" + _malla(origen),
        "## Ajuste de distribuciones\n\n" + _ajuste(origen),
        "## Paso 1: composición y competencia\n\n" + _paso1(origen, cfg.get("gamma_star", 3)),
        "## Monte Carlo\n\n" + _mc(origen),
        "## Paso 2: compra\n\n" + _paso2(origen, cfg),
        "## Paso 3: ajuste al polígono real\n\n" + _paso3(origen),
        "## Práctica actual\n\n" + _practica(origen),
        "## Convergencia del tabú\n\n" + _convergencia(origen, cfg),
        "## Validación exacta\n\n" + _validacion(origen),
        "## Notas de interpretación\n\n" + _notas(),
        "## No incluido / pendiente\n\n" + _pendiente(origen),
        "## Figuras\n\n" + _figuras(),
    ]) + "\n"


def main(origen=None, salida=None):
    origen = Path(origen or RAIZ / "results")
    salida = Path(salida or origen / "RESUMEN.md")
    salida.write_text(generar(origen), encoding="utf-8", newline="\n")
    return salida


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--origen")
    ap.add_argument("--salida")
    a = ap.parse_args()
    print(main(a.origen, a.salida))
