"""Criterio de convergencia del tabú y comparación contra la corrida de referencia."""
import sys
from pathlib import Path

import pandas as pd

try:
    from comun import RAIZ, leer_config
except ImportError:  # ejecución como paquete
    from python.comun import RAIZ, leer_config

MIN_TOTAL = 0.90
MIN_INSTANCIA = 0.80
TOL_REFERENCIA = 0.005


def _motivos(res):
    """(instancia, motivo) de cada búsqueda del pipeline; la práctica se excluye
    porque copia las búsquedas de paso 1 y paso 3."""
    res = Path(res)
    partes = []
    for archivo in ("paso1_gamma_star.csv", "paso1_barrido_gamma.csv"):
        df = pd.read_csv(res / archivo)
        for col in ("motivo_paro_paso1", "motivo_paro_opt"):
            partes.append(df[["instancia"]].assign(motivo=df[col]))
    p3 = pd.read_csv(res / "paso3_escenarios.csv")
    partes.append(p3[["instancia"]].assign(motivo=p3["motivo_paro"]))
    return pd.concat(partes, ignore_index=True)


def criterio(res):
    m = _motivos(res)
    filas = []
    for inst, g in m.groupby("instancia", sort=False):
        filas.append(_fila(inst, g.motivo))
    filas.append(_fila("total", m.motivo))
    return pd.DataFrame(filas)


def _fila(nombre, motivos):
    n = len(motivos)
    sm = int((motivos == "sin_mejora").sum())
    return dict(instancia=nombre, busquedas=n, paros_tope=int((motivos == "tope").sum()),
                paros_sin_mejora=sm, frac_sin_mejora=sm / n if n else 1.0)


def cumple(tabla):
    total = tabla[tabla.instancia == "total"].iloc[0].frac_sin_mejora
    resto = tabla[tabla.instancia != "total"].frac_sin_mejora
    return bool(total >= MIN_TOTAL and (resto >= MIN_INSTANCIA).all())


def referencia(res_final, res_ref, params_final, params_ref, n_pos,
               max_iter, max_sin_mejora):
    """Compara z_opt medio (paso 1, γ*) de la corrida final contra la de referencia."""
    filas = []
    z = {}
    en_ref = set(pd.read_csv(Path(res_ref) / "paso1_gamma_star.csv").instancia)
    for corrida, res, p in (("final", res_final, params_final),
                            ("referencia", res_ref, params_ref)):
        df = pd.read_csv(Path(res) / "paso1_gamma_star.csv")
        df = df[df.instancia.isin(en_ref)]
        for inst, g in df.groupby("instancia", sort=False):
            z[(corrida, inst)] = g.z_opt.mean()
            filas.append(dict(
                instancia=inst, corrida=corrida, candidatos=p["candidatos"],
                tenure=p["tenure"], iter_por_posicion=p["iter_por_posicion"],
                sin_mejora_por_posicion=p["sin_mejora_por_posicion"],
                max_iter_efectivo=max(p.get("max_iter", max_iter), p["iter_por_posicion"] * n_pos[inst]),
                max_sin_mejora_efectivo=max(p.get("max_sin_mejora", max_sin_mejora),
                                            p["sin_mejora_por_posicion"] * n_pos[inst]),
                z_opt_media=g.z_opt.mean(), z_opt_sd=g.z_opt.std(ddof=1),
                tiempo_medio_s=(g.tiempo_paso1_s + g.tiempo_opt_s).mean() / 2))
    for f in filas:
        zr = z[("referencia", f["instancia"])]
        f["dif_rel"] = (z[("final", f["instancia"])] - zr) / zr if f["corrida"] == "final" \
            else 0.0 if f["corrida"] == "referencia" else None
    return pd.DataFrame(filas)


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    res = Path(argv[0]) if argv else RAIZ / "results"
    tabla = criterio(res)
    tabla.to_csv(res / "convergencia_criterio.csv", index=False)
    print(tabla.to_string(index=False))
    ok = cumple(tabla)
    print("CUMPLE" if ok else "NO CUMPLE")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
