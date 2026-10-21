"""Sensibilidades: kappa, radios, m_escala y s2 sobre 1ha y map_27 (cadena completa)
y sobre map_21 (sólo paso 1 con gamma*). Cada variante vive en results/sensibilidad/<variante>/."""
import copy
import json
import shutil
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
import pandas as pd

try:
    from comun import RAIZ, RETO, leer_config
    import paso2_compra
except ImportError:
    # ejecución como paquete
    from python.comun import RAIZ, RETO, leer_config
    from python import paso2_compra

J = 10
VARIANTES = {
    "base": {},
    "kappa_0.4": {"kappa": 0.4},
    "kappa_0.8": {"kappa": 0.8},
    "radios_0.75": {"factor_radios": 0.75},
    "radios_1.25": {"factor_radios": 1.25},
    "m_0": {"m_escala": 0},
    "m_0.5": {"m_escala": 0.5},
    "m_2": {"m_escala": 2},
    "s2": {"usar_s2": True},
}
INST_CADENA = ["1ha", "map_27"]
INST_PASO1 = ["1ha", "map_27", "map_21"]


def config_variante(base, cambios):
    cfg = copy.deepcopy(base)
    cfg.update(cambios)
    cfg["escenarios_paso3_por_instancia"] = {}
    return cfg


MARCA = "completa.json"  # copia de la config con la que terminó la cadena completa


def _hecha(carpeta, cfg):
    ruta = carpeta / MARCA
    if not ruta.exists():
        return False
    try:
        return json.loads(ruta.read_text(encoding="utf-8")) == cfg
    except ValueError:
        return False


def correr_variante(nombre, carpeta, reto, ejecutar=subprocess.run,
                    mc_params=RAIZ / "results" / "mc_params.json"):
    carpeta = Path(carpeta)
    cfg = config_variante(leer_config(), VARIANTES[nombre])
    if _hecha(carpeta, cfg):
        print(f"{nombre}: ya hecha, se omite")
        return
    carpeta.mkdir(parents=True, exist_ok=True)
    (carpeta / MARCA).unlink(missing_ok=True)  # una corrida interrumpida no cuenta como hecha
    (carpeta / "config.json").write_text(json.dumps(cfg, indent=2), encoding="utf-8")
    if mc_params is not None:
        shutil.copy(mc_params, carpeta / "mc_params.json")
    comun = ["--datos", str(RAIZ / "data"), "--config", str(carpeta / "config.json"),
             "--resultados", str(carpeta)]
    cadena = ",".join(INST_CADENA)

    def reto_cmd(*args):
        ejecutar([str(reto), *args, *comun], check=True, cwd=RAIZ)

    reto_cmd("malla")
    reto_cmd("paso1", "--instancias", ",".join(INST_PASO1), "--solo-gamma-star")
    reto_cmd("montecarlo", "--instancias", cadena)
    paso2_compra.correr(resultados=carpeta)
    reto_cmd("paso3", "--instancias", cadena)
    reto_cmd("practica", "--instancias", cadena)
    (carpeta / MARCA).write_text(json.dumps(cfg, indent=2), encoding="utf-8")


def _leer(carpeta, nombre):
    ruta = carpeta / nombre
    return pd.read_csv(ruta) if ruta.exists() else None


def _fila_instancia(df, inst, **filtro):
    if df is None:
        return None
    s = df[df["instancia"] == inst]
    for k, v in filtro.items():
        s = s[np.isclose(s[k], v)] if isinstance(v, float) else s[s[k] == v]
    return s.iloc[0] if len(s) else None


def resumir(raiz):
    raiz = Path(raiz)
    nombres = [n for n in VARIANTES if (raiz / n / "paso1_gamma_star.csv").exists()]
    filas = []
    for nombre in nombres:
        car = raiz / nombre
        g = pd.read_csv(car / "paso1_gamma_star.csv")
        comp = _leer(car, "paso1_composicion.csv")
        compra = _leer(car, "paso2_compra.csv")
        p3 = _leer(car, "paso3_resumen.csv")
        pr = _leer(car, "practica_distribucion.csv")
        for inst in INST_PASO1:
            gi = g[g["instancia"] == inst]
            if gi.empty:
                continue
            f = {"variante": nombre, "instancia": inst}
            c = _fila_instancia(comp, inst)
            for j in range(J):
                f[f"D_{j}"] = c[f"D_{j}"] if c is not None else gi[f"D_{j}"].iloc[0]
            f["suma_theta_d"] = gi["suma_theta_d"].mean()
            f["z_opt_media"] = gi["z_opt"].mean()
            f["z_opt_sd"] = gi["z_opt"].std(ddof=1) if len(gi) > 1 else np.nan
            k = _fila_instancia(compra, inst, epsilon=0.1, pi_factor=2)
            f["costo_compra"] = k["costo"] if k is not None else np.nan
            f["faltante_esperado"] = k["faltante_esperado"] if k is not None else np.nan
            p = _fila_instancia(p3, inst)
            f["z_paso3_media"] = p["Z_media"] if p is not None else np.nan
            f["z_paso3_sd"] = p["Z_sd"] if p is not None else np.nan
            for col, var in [("z_practica_iii", "iii_serpentina_existentes"),
                             ("z_dsatur_con", "dsatur_con_existentes"),
                             ("z_tabu_con", "tabu_con_existentes")]:
                r = _fila_instancia(pr, inst, variante=var)
                f[col] = r["Z"] if r is not None else np.nan
            filas.append(f)
    df = pd.DataFrame(filas)
    for col, nuevo in [("z_opt_media", "cambio_z_opt_pct"), ("z_paso3_media", "cambio_z_paso3_pct")]:
        base = df[df["variante"] == "base"].set_index("instancia")[col]
        ref = df["instancia"].map(base)
        df[nuevo] = 100.0 * (df[col] - ref) / ref
    return df


def main(reto=RETO, raiz=RAIZ / "results" / "sensibilidad"):
    raiz = Path(raiz)
    for nombre in VARIANTES:
        t0 = time.time()
        correr_variante(nombre, raiz / nombre, reto)
        print(f"{nombre}: {time.time() - t0:.0f} s", flush=True)
    df = resumir(raiz)
    df.to_csv(raiz.parent / "sensibilidad.csv", index=False)
    print(f"{len(df)} filas en {raiz.parent / 'sensibilidad.csv'}")


if __name__ == "__main__":
    main(*(Path(a) for a in sys.argv[1:]))
