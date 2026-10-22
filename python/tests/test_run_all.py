"""run_all.ps1 y pasos.ps1: Invoke-Paso mide cada paso y añade una fila a tiempos.csv.
No se ejecuta run_all.ps1; sólo se comprueba que parsea."""
import shutil
import subprocess

import pandas as pd
import pytest

from comun import RAIZ

pytestmark = pytest.mark.skipif(shutil.which("powershell") is None, reason="requiere PowerShell")


def _q(p):
    return "'" + str(p).replace("'", "''") + "'"


def _ps(comando):
    return subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command",
                           comando], capture_output=True, text=True, timeout=120)


def test_invoke_paso_escribe_tiempos(tmp_path):
    ruta = tmp_path / "sub" / "tiempos.csv"
    r = _ps(f". {_q(RAIZ / 'pasos.ps1')}; "
            f"Invoke-Paso 'cmd' @('/c', 'exit', '0') -Tiempos {_q(ruta)}; "
            f"Invoke-Paso 'cmd' @('/c', 'exit 0', '--instancias', '1ha,map_27') -Tiempos {_q(ruta)}")
    assert r.returncode == 0, r.stderr
    t = pd.read_csv(ruta, dtype={"instancias": str})
    assert list(t.columns) == ["paso", "instancias", "segundos", "fecha"]
    assert len(t) == 2
    assert t.loc[0, "paso"] == "cmd /c exit 0" and pd.isna(t.loc[0, "instancias"])
    assert t.loc[1, "paso"] == "cmd /c exit 0" and t.loc[1, "instancias"] == "1ha,map_27"
    assert (t["segundos"] >= 0).all() and (t["segundos"] < 60).all()
    assert pd.to_datetime(t["fecha"]).notna().all()


def test_invoke_paso_lanza_si_falla(tmp_path):
    ruta = tmp_path / "tiempos.csv"
    r = _ps(f". {_q(RAIZ / 'pasos.ps1')}; "
            f"try {{ Invoke-Paso 'cmd' @('/c', 'exit', '3') -Tiempos {_q(ruta)}; 'NO_LANZO' }} "
            f"catch {{ 'LANZO: ' + $_.Exception.Message }}")
    assert "LANZO" in r.stdout and "NO_LANZO" not in r.stdout, r.stdout + r.stderr
    assert "codigo 3" in r.stdout
    assert not ruta.exists()


def test_run_all_parsea_y_tiene_los_pasos():
    r = _ps(f"[ScriptBlock]::Create((Get-Content -Raw {_q(RAIZ / 'run_all.ps1')})) | Out-Null; 'OK'")
    assert r.returncode == 0 and "OK" in r.stdout, r.stderr
    texto = (RAIZ / "run_all.ps1").read_text(encoding="utf-8")
    assert '. "$PSScriptRoot\\pasos.ps1"' in texto
    assert "function Invoke-Paso" not in texto
    for s in ("[switch]$OmitirValidacion", "[switch]$OmitirSensibilidad", "[switch]$SoloValidacion",
              "[string[]]$ArgumentosValidacion"):
        assert s in texto, s
    orden = ["@('practica', '--instancias', 'todas')"] + [
        f"@('python\\{s}.py')" for s in ("transicion", "entorno", "tamano", "experimentacion",
                                         "sensibilidad", "practica_compra", "markov", "figuras",
                                         "resumen")]
    pos = [texto.index(s) for s in orden]
    assert pos == sorted(pos)
    assert "@('python\\validacion_exacta.py') + $ArgumentosValidacion" in texto


def test_run_all_regenera_convergencia_antes_de_figuras_y_resumen():
    t = (RAIZ / "run_all.ps1").read_text(encoding="utf-8")
    i = t.index(r"python\convergencia.py")
    assert i < t.index(r"python\figuras.py") < t.index(r"python\resumen.py")
