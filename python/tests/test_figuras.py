from comun import RAIZ
import figuras


def test_figuras_se_crean(tmp_path):
    figuras.main(["C_por_especie", "curva_compromiso", "convergencia", "practica_compra"],
                 instancias=["1ha"], origen=RAIZ / "results", salida=tmp_path)
    for n in ["C_por_especie_1ha", "curva_compromiso_1ha", "convergencia_1ha"]:
        assert (tmp_path / f"{n}.png").stat().st_size > 1000
    assert (tmp_path / "practica_compra.png").exists()


def test_mapa_y_practica_distribucion(tmp_path):
    figuras.main(["mapa", "practica_distribucion"], instancias=["1ha"],
                 origen=RAIZ / "results", salida=tmp_path)
    assert (tmp_path / "mapa_1ha.png").stat().st_size > 1000
    assert (tmp_path / "practica_distribucion.png").stat().st_size > 1000


def test_mapa_sin_patron_se_omite_con_aviso(tmp_path, capsys):
    figuras.main(["mapa"], instancias=["no_existe"], origen=RAIZ / "results", salida=tmp_path)
    assert not (tmp_path / "mapa_no_existe.png").exists()
    assert "no_existe" in capsys.readouterr().out


def test_convergencia_incluye_map_21(tmp_path):
    figuras.main(["convergencia"], origen=RAIZ / "results", salida=tmp_path)
    for n in ["convergencia_1ha", "convergencia_map_27", "convergencia_map_21"]:
        assert (tmp_path / f"{n}.png").stat().st_size > 1000
    figuras.main(["curva_compromiso"], instancias=["1ha"], origen=RAIZ / "results", salida=tmp_path)
    assert not (tmp_path / "curva_compromiso_map_21.png").exists()


def test_practica_distribucion_con_siete_variantes(tmp_path, monkeypatch):
    import pandas as pd
    variantes = ["i_serpentina", "ii_serpentina_banda", "iii_serpentina_existentes",
                 "tabu_sin_existentes", "tabu_con_existentes",
                 "dsatur_sin_existentes", "dsatur_con_existentes"]
    filas = []
    for inst in ["1ha", "map_27"]:
        for k, v in enumerate(variantes):
            niv = [10.0 + k] * 5 + [1.0]
            filas.append([inst, v, 100, sum(niv), sum(niv) / 100, k] + niv + [""])
    cols = ["instancia", "variante", "plantas", "Z", "Z_por_planta",
            "pares_inmediatos_misma_especie"] + [f"Z_nivel{i}" for i in range(1, 6)] + \
           ["Z_otro", "motivo_paro"]
    pd.DataFrame(filas, columns=cols).to_csv(tmp_path / "practica_distribucion.csv", index=False)
    figs = []
    original = figuras._guardar
    monkeypatch.setattr(figuras, "_guardar", lambda fig, ruta: (figs.append(fig), original(fig, ruta)))
    salida = tmp_path / "fig"
    salida.mkdir()
    figuras.main(["practica_distribucion"], origen=tmp_path, salida=salida)
    assert (salida / "practica_distribucion.png").stat().st_size > 1000
    etiquetas = [t.get_text() for t in figs[0].axes[0].get_xticklabels()]
    assert len(etiquetas) == 7
    assert any("DSATUR" in e for e in etiquetas)
    assert len(figs[0].axes[0].patches) == 7
