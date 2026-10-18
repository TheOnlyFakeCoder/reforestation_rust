from comun import RAIZ
import figuras


def test_figuras_se_crean(tmp_path):
    figuras.main(["C_por_especie", "curva_compromiso", "convergencia", "practica_compra"],
                 instancias=["1ha"], origen=RAIZ / "results", salida=tmp_path)
    for n in ["C_por_especie_1ha", "curva_compromiso_1ha", "convergencia_1ha"]:
        assert (tmp_path / f"{n}.png").stat().st_size > 1000
    assert (tmp_path / "practica_compra.png").exists()
