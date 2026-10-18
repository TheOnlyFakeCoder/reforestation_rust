from comun import leer_config, leer_poligonos, leer_especies


def test_poligonos_30_filas_y_10_especies():
    df = leer_poligonos()
    assert len(df) == 30
    assert [c for c in df.columns if c.startswith("X_")] == [f"X_{j}" for j in range(10)]


def test_especies_10():
    assert len(leer_especies()) == 10


def test_config_dict():
    cfg = leer_config()
    assert isinstance(cfg, dict) and "d" in cfg
