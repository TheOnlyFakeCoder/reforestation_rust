import json

import entorno

HASH = "a" * 40


def falso(cmd):
    texto = " ".join(cmd)
    if cmd[0] == "git":
        return HASH + "\n"
    if cmd[0] in ("rustc", "cargo"):
        return f"{cmd[0]} 1.90.0 (abc 2026-01-01)\n"
    if "Win32_ComputerSystem" in texto:
        return json.dumps({"Manufacturer": "ACME", "Model": "Laptop X",
                           "TotalPhysicalMemory": 34359738368})
    if "Win32_Processor" in texto:
        return json.dumps({"Name": "CPU Z", "NumberOfCores": 24,
                           "NumberOfLogicalProcessors": 32})
    if "Win32_OperatingSystem" in texto:
        return json.dumps({"Caption": "Microsoft Windows 11", "Version": "10.0.26300"})
    if "Win32_LogicalDisk" in texto:
        return json.dumps({"Size": 1073741824000, "FreeSpace": 268435456000})
    if "Win32_PhysicalMemory" in texto:
        return json.dumps([{"Capacity": 17179869184, "Speed": 5600, "SMBIOSMemoryType": 34},
                           {"Capacity": 17179869184, "Speed": 5600, "SMBIOSMemoryType": 34}])
    if "Get-PhysicalDisk" in texto:
        return json.dumps({"FriendlyName": "NVMe", "MediaType": "SSD"})
    raise RuntimeError("comando inesperado")


def test_recolectar_con_salidas_falsas():
    r = entorno.recolectar(ejecutar=falso)
    assert r["nucleos_fisicos"] == 24
    assert r["hilos_logicos"] == 32
    assert r["ram_gb"] == 32.0
    assert len(r["ram_modulos"]) == 2
    assert all(m["tipo"] == "DDR5" and m["velocidad_mhz"] == 5600 and m["capacidad_gb"] == 16.0
               for m in r["ram_modulos"])
    assert r["tipo_disco"] == "SSD"
    assert r["disco_total_gb"] == 1000.0 and r["disco_libre_gb"] == 250.0
    assert r["modelo_pc"] == "ACME Laptop X"
    assert r["procesador"] == "CPU Z"
    assert "Windows 11" in r["sistema_operativo"]
    assert r["commit"] == HASH
    assert set(r["software"]) >= {"rustc", "cargo", "python", "numpy", "scipy", "pandas",
                                  "matplotlib", "pulp", "highspy", "ortools", "pytest"}
    assert r["software"]["numpy"] != ""


def test_recolectar_tolera_comando_fallido():
    def malo(cmd):
        raise OSError("no existe")
    r = entorno.recolectar(ejecutar=malo)
    for k in ("modelo_pc", "sistema_operativo", "procesador", "nucleos_fisicos",
              "hilos_logicos", "ram_gb", "tipo_disco", "commit", "disco_total_gb"):
        assert r[k] == "desconocido"
    assert r["ram_modulos"] == []
    assert r["software"]["rustc"] == "desconocido"


def test_tipo_ram_desconocido_y_modulo_unico():
    def f(cmd):
        if "Win32_PhysicalMemory" in " ".join(cmd):
            return json.dumps({"Capacity": 8589934592, "Speed": 1600, "SMBIOSMemoryType": 99})
        return falso(cmd)
    r = entorno.recolectar(ejecutar=f)
    assert r["ram_modulos"][0]["tipo"] == "código 99"


def test_escribir_md_y_json(tmp_path):
    r = entorno.recolectar(ejecutar=falso)
    entorno.escribir(r, tmp_path)
    assert json.loads((tmp_path / "entorno.json").read_text(encoding="utf-8"))["commit"] == HASH
    assert "DDR5" in (tmp_path / "entorno.md").read_text(encoding="utf-8")
