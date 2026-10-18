"""Ajuste de distribuciones de densidad y composición (spec §4)."""
import json
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from scipy import optimize, stats
from scipy.special import gammaln

from comun import RAIZ, leer_especies, leer_poligonos


def densidad(area, X):
    return X.sum(axis=1) / area


def sigma_mle(rho):
    return float(rho.std(ddof=0))


def p_hat(X):
    return X.sum(axis=0) / X.sum()


def ajustar_densidad(rho):
    n = len(rho)
    filas = []
    modelos = [
        ("Normal", stats.norm, {}),
        ("Gamma", stats.gamma, {"floc": 0}),
        ("Lognormal", stats.lognorm, {"floc": 0}),
    ]
    for nombre, dist, kw in modelos:
        par = dist.fit(rho, **kw)
        ll = float(dist.logpdf(rho, *par).sum())
        k = len(par) - len(kw)
        ks = stats.kstest(rho, dist.cdf, args=par).pvalue
        filas.append({
            "modelo": nombre,
            "parametros": " ".join(f"{v:.6f}" for v in par),
            "loglik": ll,
            "AIC": 2 * k - 2 * ll,
            "KS_p": float(ks),
        })
    return pd.DataFrame(filas)


def loglik_dm(X, alpha):
    """Log-verosimilitud Dirichlet-multinomial (sin el coeficiente multinomial)."""
    T = X.sum(axis=1)
    a0 = alpha.sum()
    return float(
        (gammaln(a0) - gammaln(T + a0)).sum()
        + (gammaln(X + alpha) - gammaln(alpha)).sum()
    )


def precision_dm(X):
    p = p_hat(X)
    f = lambda la: -loglik_dm(X, np.exp(la) * p)
    r = optimize.minimize_scalar(f, bounds=(np.log(1.0), np.log(1e9)), method="bounded",
                                 options={"xatol": 1e-10})
    return float(np.exp(r.x))


def resumen(area, X):
    rho = densidad(area, X)
    T = X.sum(axis=1)
    mu_pool = T.sum() / area.sum()
    esperado = mu_pool * area
    disp = float(((T - esperado) ** 2 / esperado).sum() / (len(T) - 1))
    chi2, p, gl, _ = stats.chi2_contingency(X.T)
    a0 = precision_dm(X)
    ph = p_hat(X)
    return {
        "n_poligonos": len(rho),
        "media_rho": float(rho.mean()),
        "sd_rho": float(rho.std(ddof=1)),
        "sigma_mle": sigma_mle(rho),
        "shapiro_p": float(stats.shapiro(rho).pvalue),
        "dispersion_poisson": disp,
        "corr_rho_area": float(np.corrcoef(rho, area)[0, 1]),
        "chi2_homog": float(chi2),
        "chi2_gl": int(gl),
        "chi2_p": float(p),
        "precision_dm": a0,
        "loglik_dm": loglik_dm(X, a0 * ph),
        "loglik_multinomial": float((X * np.log(ph)).sum()),
    }


def _ll_bn(k, x, m):
    return float(stats.nbinom.logpmf(x, k, k / (k + m)).sum())


def por_especie(area, X, nombres=None):
    filas = []
    for j in range(X.shape[1]):
        x = X[:, j]
        mu = x.sum() / area.sum()
        m = mu * area
        disp = float(((x - m) ** 2 / m).sum() / (len(x) - 1))
        ll_p = float(stats.poisson.logpmf(x, m).sum())
        r = optimize.minimize_scalar(lambda lk: -_ll_bn(np.exp(lk), x, m),
                                     bounds=(np.log(1e-2), np.log(1e8)), method="bounded",
                                     options={"xatol": 1e-10})
        k = float(np.exp(r.x))
        ll_b = -float(r.fun)
        filas.append({
            "especie": nombres[j] if nombres is not None else j,
            "mu_ha": float(mu),
            "dispersion_poisson": disp,
            "loglik_poisson": ll_p,
            "AIC_poisson": 2 - 2 * ll_p,
            "k_bn": k,
            "loglik_bn": ll_b,
            "AIC_bn": 4 - 2 * ll_b,
        })
    return pd.DataFrame(filas)


def figura_hist(rho, tabla, ruta):
    fig, ax = plt.subplots(figsize=(7, 4.2))
    ax.hist(rho, bins=10, density=True, color="#bbbbbb", edgecolor="white", label="datos")
    xs = np.linspace(rho.min() - 5, rho.max() + 5, 400)
    dists = {"Normal": stats.norm, "Gamma": stats.gamma, "Lognormal": stats.lognorm}
    for (_, f), c in zip(tabla.iterrows(), ["#1b6ca8", "#d95f02", "#1b9e77"]):
        par = [float(v) for v in f["parametros"].split()]
        ax.plot(xs, dists[f["modelo"]].pdf(xs, *par), color=c, lw=2, label=f["modelo"])
    ax.set_xlabel("densidad de plantas existentes ρ (plantas/ha)")
    ax.set_ylabel("densidad")
    ax.legend()
    fig.tight_layout()
    fig.savefig(ruta, dpi=300)
    plt.close(fig)


def main(resultados=None, figuras=None, datos=None):
    resultados = Path(resultados or RAIZ / "results")
    figuras = Path(figuras or RAIZ / "figures")
    resultados.mkdir(parents=True, exist_ok=True)
    figuras.mkdir(parents=True, exist_ok=True)
    df = leer_poligonos(datos)
    area = df["area_ha"].to_numpy()
    X = df[[f"X_{j}" for j in range(10)]].to_numpy(dtype=float)
    rho = densidad(area, X)
    tabla = ajustar_densidad(rho)
    tabla.to_csv(resultados / "ajuste_densidad.csv", index=False, float_format="%.6f")
    res = resumen(area, X)
    pd.DataFrame({"estadistico": list(res), "valor": list(res.values())}).to_csv(
        resultados / "ajuste_resumen.csv", index=False, float_format="%.6f")
    esp = leer_especies(datos)["especie"].tolist()
    por_especie(area, X, esp).to_csv(resultados / "ajuste_por_especie.csv", index=False,
                                     float_format="%.6f")
    mc = {"mu": float(rho.mean()), "sigma": sigma_mle(rho), "p_hat": p_hat(X).tolist()}
    (resultados / "mc_params.json").write_text(json.dumps(mc, indent=2), encoding="utf-8")
    figura_hist(rho, tabla, figuras / "hist_rho.png")
    return res


if __name__ == "__main__":
    for k, v in main().items():
        print(k, v)
