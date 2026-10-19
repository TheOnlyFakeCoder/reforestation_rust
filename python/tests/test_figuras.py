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
