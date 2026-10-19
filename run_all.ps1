# Pipeline completo del Entregable 2 (ver README.md). Ejecutar desde la raíz del repositorio.
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

$reto = Join-Path $PSScriptRoot 'target\release\reto.exe'
$py = Join-Path $PSScriptRoot '.venv\Scripts\python.exe'

function Invoke-Paso {
    param([string]$Exe, [string[]]$Argumentos)
    Write-Host ">> $Exe $($Argumentos -join ' ')"
    $previo = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'  # cargo escribe progreso por stderr
    & $Exe @Argumentos
    $codigo = $LASTEXITCODE
    $ErrorActionPreference = $previo
    if ($codigo -ne 0) { throw "fallo: $Exe $($Argumentos -join ' ') (codigo $codigo)" }
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
Invoke-Paso $py @('python\practica_compra.py')
Invoke-Paso $py @('python\markov.py')
Invoke-Paso $py @('python\figuras.py')
Invoke-Paso $py @('python\resumen.py')
Write-Host 'Listo: results/RESUMEN.md y figures/'
