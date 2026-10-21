"""Tamaño del problema: variables, restricciones, parámetros y datos por instancia."""
from pathlib import Path

import numpy as np
import pandas as pd

try:
    from comun import RAIZ, leer_config, leer_especies
    from paso2_compra import escenarios_C, modelo_ple
except ImportError:  # ejecución como paquete
    from python.comun import RAIZ, leer_config, leer_especies
    from python.paso2_compra import escenarios_C, modelo_ple

INSTANCIAS = ("1ha", "map_27")
J = 10
N_OMEGA = 1000


def _fila(instancia, modelo, variables, tipo, restricciones, nota):
    return dict(instancia=instancia, modelo=modelo, variables=int(variables),
                tipo_variables=tipo, restricciones=int(restricciones), nota=nota)


def _contar(x):
    """len() de un atributo de PuLP que según la versión es método o propiedad."""
    return len(x() if callable(x) else x)


def _aristas(m):
    return int(sum(m[c] for c in m.index if str(c).startswith("pares_anillo_")))


def tamano_modelos(origen):
    origen = Path(origen)
    cfg = leer_config()
    c = leer_especies()["c"].to_numpy(dtype=float)
    malla = pd.read_csv(origen / "malla_resumen.csv").set_index("instancia")
    comp = pd.read_csv(origen / "paso1_composicion.csv").set_index("instancia")
    p3 = pd.read_csv(origen / "paso3_escenarios.csv")
    filas = []
    for inst in INSTANCIAS:
        m = malla.loc[inst]
        V, E = int(m["n_pos"]), _aristas(m)
        x_y = 10 * V + 100 * E
        restr_lin = V + 10 + 20 + 3 * 100 * E

        filas.append(_fila(inst, "paso1_tabu", V, "enteras en {0..10}", 10 + 20,
                           "una variable por posición (la asignación única va implícita en "
                           "phi_u en {0..10}); restricciones: 10 definiciones D_j = suma_u [phi_u = j] "
                           "y 20 cotas L_j <= D_j <= U_j, sin restricciones lineales en la búsqueda "
                           "tabú en sí"))
        filas.append(_fila(inst, "paso1_lineal", x_y, "binarias",
                           restr_lin,
                           f"10*|V| binarias x + 100*|E| y, |V|={V}, |E|={E}; "
                           "restricciones |V|+10+20+3*100*|E|"))

        r = comp.loc[inst]
        D = np.array([r[f"D_{j}"] for j in range(J)], dtype=float)
        X = pd.read_csv(origen / f"escenarios_X_{inst}.csv")[
            [f"X_{j}" for j in range(J)]].to_numpy(dtype=float)
        prob, q = modelo_ple(escenarios_C(D, X), c, cfg["pi_factor"],
                             presupuesto=cfg["presupuesto_ha"] * float(r["area_ha"]))
        n_ple = _contar(prob.variables)
        filas.append(_fila(inst, "paso2_ple", n_ple,
                           f"{len(q)} enteras q + {n_ple - len(q)} continuas f/e",
                           _contar(prob.constraints),
                           "conteos reales del modelo agrupado por valores distintos de "
                           f"C_j; la forma completa sin agrupar tiene 10 q + 2*10*|Omega| "
                           f"f/e (|Omega|={len(X)} en esta corrida)"))

        v0 = int(p3[(p3.instancia == inst) & (p3.omega == 0)].iloc[0]["V_lib"])
        filas.append(_fila(inst, "paso3_por_escenario", v0, "enteras en {0..10}", 10,
                           f"posiciones libres |V_lib| del escenario omega=0; restricciones: "
                           f"10 cuotas n_j; se resuelve una vez por cada uno de los |Omega|={N_OMEGA}"))

        var_ext = x_y + 10 + N_OMEGA * (x_y + 20)
        restr_ext = restr_lin + 1 + N_OMEGA * (V + 10 + 3 * 100 * E + 20)
        filas.append(_fila(inst, "extensivo_completo", var_ext, "binarias y enteras",
                           restr_ext,
                           f"restricciones: las del paso 1 + 1 restricción de presupuesto + por "
                           f"cada omega las del acomodo y de f_w, e_w; "
                           f"x, y del paso 1 + q (10) + por cada omega de |Omega|={N_OMEGA}: "
                           "x_w, y_w del acomodo (10*|V|+100*|E|) y f_w, e_w (20); "
                           "justifica la descomposición en pasos"))
    return pd.DataFrame(filas, columns=["instancia", "modelo", "variables",
                                        "tipo_variables", "restricciones", "nota"])


def parametros():
    n = len(leer_especies())
    filas = [
        ("costo unitario", "c_j", n, "data/especies.csv"),
        ("supervivencia a 1 año", "s1_j", n, "data/supervivencia.csv"),
        ("supervivencia a 2 años", "s2_j", n, "data/supervivencia.csv"),
        ("densidad nominal", "δ_j", n, "data/especies.csv"),
        ("radio de copa", "R_j", n, "data/rasgos.csv (radio_d)"),
        ("rasgos (7 por especie)", "rasgos", 7 * n, "data/rasgos.csv"),
        ("matriz de interacción", "W", n * n, "construida de rasgos"),
        ("umbral de poda", "θ_j", n, "derivado de c, s1, W (config m_escala)"),
        ("cota inferior de composición", "L_j", n, "paso 1 (ε)"),
        ("cota superior de composición", "U_j", n, "paso 1 (ε)"),
        ("penalización por faltante", "π_j", n, "pi_factor * c_j"),
        ("distancia de vecindad", "d", 1, "config/default.json"),
        ("tolerancia de composición", "ε", 1, "config/default.json"),
        ("factor de vecindad", "κ", 1, "config/default.json"),
        ("peso de interacción elegido", "γ*", 1, "results/paso1_gamma_star.csv"),
        ("factor de radios", "factor_radios", 1, "config/default.json"),
        ("presupuesto por ha", "B", 1, "config/default.json"),
        ("penalización por sobrante", "η", 1, "config (0 por defecto)"),
        ("media de plantas Monte Carlo", "μ_ρ", 1, "results/mc_params.json"),
        ("desviación de plantas Monte Carlo", "σ_ρ", 1, "results/mc_params.json"),
        ("proporciones de especie", "p̂_j", n, "results/mc_params.json"),
        ("escenarios Monte Carlo", "|Ω|", 1, "config/default.json"),
        ("escenarios del paso 3", "|Ω₃|", 1, "config/default.json"),
    ]
    df = pd.DataFrame(filas, columns=["parametro", "simbolo", "cantidad", "fuente"])
    total = pd.DataFrame([("TOTAL", "", int(df.cantidad.sum()), "suma")],
                         columns=df.columns)
    return pd.concat([df, total], ignore_index=True)


def problema(origen):
    origen = Path(origen)
    malla = pd.read_csv(origen / "malla_resumen.csv").set_index("instancia")
    comp = pd.read_csv(origen / "paso1_composicion.csv").set_index("instancia")
    filas = []
    for inst in INSTANCIAS:
        r = comp.loc[inst]
        fila = {"instancia": inst, "especies": J,
                "plantas_por_ha": float(r["d_tot"]) / float(r["area_ha"]),
                "D_tot": int(r["d_tot"]), "n_pos": int(malla.loc[inst, "n_pos"])}
        fila.update({f"D_{j}": int(r[f"D_{j}"]) for j in range(J)})
        fila.update({f"Dnom_{j}": int(r[f"Dnom_{j}"]) for j in range(J)})
        filas.append(fila)
    return pd.DataFrame(filas)


def main(origen=None, destino=None):
    origen = Path(origen or RAIZ / "results")
    destino = Path(destino or RAIZ / "results")
    destino.mkdir(parents=True, exist_ok=True)
    tamano_modelos(origen).to_csv(destino / "tamano_modelo.csv", index=False)
    parametros().to_csv(destino / "tamano_parametros.csv", index=False)
    problema(origen).to_csv(destino / "tamano_problema.csv", index=False)


if __name__ == "__main__":
    main()
