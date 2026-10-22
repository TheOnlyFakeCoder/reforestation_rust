"""Características de la PC, versiones de software y commit (results/entorno.*)."""
import json
import platform
import subprocess
from importlib import metadata
from pathlib import Path

try:
    from comun import RAIZ
except ImportError:  # ejecución como paquete
    from python.comun import RAIZ

DESC = "desconocido"
TIPOS_RAM = {20: "DDR", 21: "DDR2", 24: "DDR3", 26: "DDR4", 34: "DDR5", 35: "LPDDR5"}
PAQUETES = ("numpy", "scipy", "pandas", "matplotlib", "pulp", "highspy", "ortools", "pytest")


def _ejecutar(cmd):
    """Corre un comando y devuelve su salida estándar; lanza si falla."""
    p = subprocess.run(cmd, capture_output=True, text=True, timeout=60, cwd=RAIZ)
    if p.returncode != 0:
        raise RuntimeError(p.stderr.strip() or f"código {p.returncode}")
    return p.stdout


def _ps(ejecutar, consulta):
    """Consulta PowerShell -> lista de dicts ([] si falla)."""
    try:
        txt = ejecutar(["powershell", "-NoProfile", "-Command", f"{consulta} | ConvertTo-Json"])
        datos = json.loads(txt)
    except Exception:
        return []
    if isinstance(datos, dict):
        return [datos]
    return [d for d in datos if isinstance(d, dict)] if isinstance(datos, list) else []


def _redondear(valor, escala, dec=1):
    try:
        return round(float(valor) / escala, dec)
    except (TypeError, ValueError):
        return DESC


def _tipo_ram(codigo):
    try:
        n = int(codigo)
    except (TypeError, ValueError):
        return DESC
    return TIPOS_RAM.get(n, f"código {n}")


def _version_cmd(ejecutar, cmd):
    try:
        return ejecutar(cmd).strip().splitlines()[0].strip() or DESC
    except Exception:
        return DESC


def _version_paquete(nombre):
    try:
        return metadata.version(nombre)
    except Exception:
        return DESC


def recolectar(ejecutar=_ejecutar):
    sis = (_ps(ejecutar, "Get-CimInstance Win32_ComputerSystem") or [{}])[0]
    cpu = (_ps(ejecutar, "Get-CimInstance Win32_Processor") or [{}])[0]
    so = (_ps(ejecutar, "Get-CimInstance Win32_OperatingSystem") or [{}])[0]
    disco = (_ps(ejecutar,
                 "Get-CimInstance Win32_LogicalDisk -Filter \"DeviceID='C:'\"") or [{}])[0]
    mods = _ps(ejecutar, "Get-CimInstance Win32_PhysicalMemory")
    fisicos = _ps(ejecutar, "Get-PhysicalDisk")

    fab, mod = sis.get("Manufacturer"), sis.get("Model")
    modelo = " ".join(str(x).strip() for x in (fab, mod) if x) or DESC
    so_txt = (f"{so['Caption'].strip()} (versión {so.get('Version', '?')})"
              if so.get("Caption") else DESC)
    tipos = sorted({str(d.get("MediaType")) for d in fisicos if d.get("MediaType")})
    try:
        commit = ejecutar(["git", "rev-parse", "HEAD"]).strip() or DESC
    except Exception:
        commit = DESC

    software = {"rustc": _version_cmd(ejecutar, ["rustc", "--version"]),
                "cargo": _version_cmd(ejecutar, ["cargo", "--version"]),
                "python": platform.python_version()}
    for p in PAQUETES:
        software[p] = _version_paquete(p)

    return {
        "modelo_pc": modelo,
        "sistema_operativo": so_txt,
        "disco_total_gb": _redondear(disco.get("Size"), 2 ** 30),
        "disco_libre_gb": _redondear(disco.get("FreeSpace"), 2 ** 30),
        "tipo_disco": "/".join(tipos) if tipos else DESC,
        "ram_gb": _redondear(sis.get("TotalPhysicalMemory"), 2 ** 30),
        "ram_modulos": [{"capacidad_gb": _redondear(m.get("Capacity"), 2 ** 30),
                         "tipo": _tipo_ram(m.get("SMBIOSMemoryType")),
                         "velocidad_mhz": m.get("Speed", DESC)} for m in mods],
        "procesador": str(cpu["Name"]).strip() if cpu.get("Name") else DESC,
        "nucleos_fisicos": cpu.get("NumberOfCores", DESC),
        "hilos_logicos": cpu.get("NumberOfLogicalProcessors", DESC),
        "commit": commit,
        "software": software,
    }


def a_markdown(r):
    L = ["# Entorno de ejecución", "",
         f"- Equipo: {r['modelo_pc']}",
         f"- Sistema operativo: {r['sistema_operativo']}",
         f"- Procesador: {r['procesador']} ({r['nucleos_fisicos']} núcleos físicos, "
         f"{r['hilos_logicos']} hilos lógicos)",
         f"- RAM total: {r['ram_gb']} GB",
         f"- Disco C: {r['disco_total_gb']} GB totales, {r['disco_libre_gb']} GB libres "
         f"({r['tipo_disco']})",
         f"- Commit: {r['commit']}", "", "## Módulos de RAM", "",
         "| Capacidad (GB) | Tipo | Velocidad (MHz) |", "|---|---|---|"]
    for m in r["ram_modulos"]:
        L.append(f"| {m['capacidad_gb']} | {m['tipo']} | {m['velocidad_mhz']} |")
    L += ["", "## Software", "", "| Componente | Versión |", "|---|---|"]
    L += [f"| {k} | {v} |" for k, v in r["software"].items()]
    return "\n".join(L) + "\n"


def escribir(r, destino):
    destino = Path(destino)
    destino.mkdir(parents=True, exist_ok=True)
    (destino / "entorno.json").write_text(
        json.dumps(r, ensure_ascii=False, indent=2), encoding="utf-8")
    (destino / "entorno.md").write_text(a_markdown(r), encoding="utf-8")


def main(destino=None):
    r = recolectar()
    escribir(r, destino or RAIZ / "results")
    return r


if __name__ == "__main__":
    print(json.dumps(main(), ensure_ascii=False, indent=2))
