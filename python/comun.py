"""Lectura de datos y configuración compartida por los scripts de Python."""
import json
import os
from pathlib import Path

import pandas as pd

RAIZ = Path(__file__).resolve().parents[1]

# Binario del CLI: reto.exe en Windows, reto en el resto (cargo build --release --bin reto).
RETO = RAIZ / "target" / "release" / ("reto.exe" if os.name == "nt" else "reto")


def leer_poligonos(datos=None):
    return pd.read_csv(Path(datos or RAIZ / "data") / "poligonos.csv")


def leer_especies(datos=None):
    return pd.read_csv(Path(datos or RAIZ / "data") / "especies.csv")


def leer_config(ruta=None):
    with open(ruta or RAIZ / "config" / "default.json", encoding="utf-8") as f:
        return json.load(f)
