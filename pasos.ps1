# Invoke-Paso: corre un paso del pipeline, lanza si su codigo de salida no es 0 y, si termina bien,
# anade una fila a tiempos.csv (paso, instancias, segundos, fecha) medida con Measure-Command.
# "instancias" es el valor de --instancias (vacio si no lo hay) y "paso" el ejecutable con el resto
# de los argumentos. La cabecera se escribe si el archivo no existe. Lo carga run_all.ps1.

function Invoke-Paso {
    param([string]$Exe, [string[]]$Argumentos = @(), [string]$Tiempos = 'results\tiempos.csv')
    Write-Host ">> $Exe $($Argumentos -join ' ')"
    $previo = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'  # cargo escribe progreso por stderr
    try {
        $medida = Measure-Command { & $Exe @Argumentos | Out-Host }
        $codigo = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previo
    }
    if ($codigo -ne 0) { throw "fallo: $Exe $($Argumentos -join ' ') (codigo $codigo)" }

    $instancias = ''
    $resto = @()
    for ($i = 0; $i -lt $Argumentos.Count; $i++) {
        if ($Argumentos[$i] -eq '--instancias' -and $i + 1 -lt $Argumentos.Count) {
            $instancias = $Argumentos[$i + 1]
            $i++
        } else {
            $resto += $Argumentos[$i]
        }
    }
    $nombre = [IO.Path]::GetFileNameWithoutExtension($Exe)
    $paso = (@($nombre) + $resto) -join ' '
    $inv = [Globalization.CultureInfo]::InvariantCulture
    $segundos = $medida.TotalSeconds.ToString('0.###', $inv)
    $fecha = (Get-Date).ToString('yyyy-MM-ddTHH:mm:ss', $inv)

    $ruta = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Tiempos)
    $dir = Split-Path -Parent $ruta
    if ($dir -and -not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
    $utf8 = New-Object System.Text.UTF8Encoding $false
    if (-not (Test-Path $ruta)) {
        [IO.File]::WriteAllText($ruta, "paso,instancias,segundos,fecha`n", $utf8)
    }
    $campo = { param($s) '"' + ($s -replace '"', '""') + '"' }
    $linea = "$(& $campo $paso),$(& $campo $instancias),$segundos,$fecha`n"
    [IO.File]::AppendAllText($ruta, $linea, $utf8)
}
