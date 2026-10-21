# Pipeline completo del Entregable 2 (ver README.md). Ejecutar desde la raiz del repositorio.
# -OmitirValidacion salta la validacion exacta con CP-SAT (hasta ~10 min por instancia y formulacion).
# -OmitirSensibilidad salta python\sensibilidad.py (9 variantes de la cadena completa en 1ha y P1).
# -SoloValidacion corre solo los tres pasos de la validacion exacta (requiere reto.exe compilado).
# -ArgumentosValidacion se pasa a python\validacion_exacta.py, p. ej.
#   -ArgumentosValidacion @('--formulaciones', 'ajustada', '--workers', '32', '--actualizar')
# Cada paso se mide con Measure-Command y anade una fila a results\tiempos.csv (ver pasos.ps1).
param([switch]$OmitirValidacion, [switch]$OmitirSensibilidad, [switch]$SoloValidacion,
      [string[]]$ArgumentosValidacion = @())
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
. "$PSScriptRoot\pasos.ps1"

$reto = Join-Path $PSScriptRoot 'target\release\reto.exe'
$py = Join-Path $PSScriptRoot '.venv\Scripts\python.exe'

function Invoke-Validacion {
    Invoke-Paso $reto @('instancias-val')
    Invoke-Paso $reto @('tabu-val')
    Invoke-Paso $py (@('python\validacion_exacta.py') + $ArgumentosValidacion)
}

if ($SoloValidacion) {
    Invoke-Validacion
    Write-Host 'Listo: results/validacion_exacta.csv'
    return
}

Invoke-Paso 'cargo' @('build', '--release', '--bin', 'reto')
Invoke-Paso $reto @('datos')
Invoke-Paso $reto @('malla')
Invoke-Paso $py @('python\ajuste.py')
Invoke-Paso $reto @('paso1', '--instancias', '1ha,map_27')
Invoke-Paso $reto @('paso1', '--instancias', 'todas', '--solo-gamma-star')
Invoke-Paso $reto @('montecarlo', '--instancias', 'todas')
Invoke-Paso $py @('python\paso2_compra.py')
Invoke-Paso $reto @('paso3', '--instancias', 'todas')
Invoke-Paso $reto @('practica', '--instancias', 'todas')
Invoke-Paso $py @('python\transicion.py')
Invoke-Paso $py @('python\entorno.py')
Invoke-Paso $py @('python\tamano.py')
Invoke-Paso $py @('python\experimentacion.py')
if (-not $OmitirSensibilidad) {
    Invoke-Paso $py @('python\sensibilidad.py')
}
if (-not $OmitirValidacion) {
    Invoke-Validacion
}
Invoke-Paso $py @('python\practica_compra.py')
Invoke-Paso $py @('python\markov.py')
Invoke-Paso $py @('python\convergencia.py')
Invoke-Paso $py @('python\figuras.py')
Invoke-Paso $py @('python\resumen.py')
Write-Host 'Listo: results/RESUMEN.md y figures/'
