import json

import numpy as np
import pytest

import ajuste
from comun import leer_poligonos


@pytest.fixture(scope="module")
def datos():
    df = leer_poligonos()
    area = df["area_ha"].to_numpy()
    X = df[[f"X_{j}" for j in range(10)]].to_numpy(dtype=float)
    return area, X


def test_rho(datos):
    area, X = datos
    rho = ajuste.densidad(area, X)
    assert rho.shape == (30,)
    assert np.allclose(rho, X.sum(axis=1) / area)


def test_ajuste_densidad_valores(datos):
    area, X = datos
    rho = ajuste.densidad(area, X)
    tabla = ajuste.ajustar_densidad(rho)
    assert list(tabla["modelo"]) == ["Normal", "Gamma", "Lognormal"]
    assert round(rho.mean(), 2) == 129.18
    assert round(rho.std(ddof=1), 2) == 3.35
    assert round(ajuste.sigma_mle(rho), 2) == 3.29
    aic = dict(zip(tabla["modelo"], tabla["AIC"]))
    assert round(aic["Normal"], 2) == 160.58
    assert round(aic["Gamma"], 2) == 160.40
    assert round(aic["Lognormal"], 2) == 160.32
    assert ((tabla["KS_p"] > 0) & (tabla["KS_p"] <= 1)).all()


def test_resumen(datos):
    area, X = datos
    r = ajuste.resumen(area, X)
    assert round(r["shapiro_p"], 2) == 0.21
    assert round(r["dispersion_poisson"], 2) == 0.41
    assert round(r["corr_rho_area"], 2) == -0.38
    assert round(r["chi2_homog"], 1) == 286.7
    assert r["chi2_gl"] == 261
    assert round(r["chi2_p"], 2) == 0.13


def test_p_hat(datos):
    _, X = datos
    p = ajuste.p_hat(X)
    assert p.shape == (10,) and abs(p.sum() - 1) < 1e-12
    assert np.allclose(p, X.sum(axis=0) / X.sum())


def test_dirichlet_multinomial_mejora_loglik(datos):
    _, X = datos
    p = ajuste.p_hat(X)
    a0 = ajuste.precision_dm(X)
    assert 14000 < a0 < 16500
    assert round(a0) == 15277
    ll = lambda a: ajuste.loglik_dm(X, a * p)
    assert ll(a0) >= ll(a0 * 0.5) and ll(a0) >= ll(a0 * 2)


def test_por_especie(datos):
    area, X = datos
    t = ajuste.por_especie(area, X)
    assert len(t) == 10
    assert {"especie", "mu_ha", "dispersion_poisson", "AIC_poisson", "k_bn", "AIC_bn"} <= set(t.columns)
    d = t["dispersion_poisson"]
    assert round(d.min(), 2) == 0.58 and round(d.max(), 2) == 1.45


def test_main_escribe_salidas(tmp_path):
    ajuste.main(resultados=tmp_path, figuras=tmp_path)
    mc = json.loads((tmp_path / "mc_params.json").read_text())
    assert round(mc["mu"], 2) == 129.18 and round(mc["sigma"], 2) == 3.29
    assert len(mc["p_hat"]) == 10
    for f in ["ajuste_densidad.csv", "ajuste_resumen.csv", "ajuste_por_especie.csv", "hist_rho.png"]:
        assert (tmp_path / f).exists()
