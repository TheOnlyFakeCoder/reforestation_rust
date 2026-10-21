# Adenda de la spec — Fase 5 (entrega final: "Experimentación y resultados")

Complementa `2026-10-16-entregable2-spec.md`; lo que no se dice aquí sigue esa spec.

## Lo que pide la entrega final (texto del curso)

- Características de la PC y software: modelo de la PC, sistema operativo, capacidad de disco duro,
  memoria RAM y capacidad, tipo de procesador, número de núcleos, software utilizado y versión.
- Tamaño del problema: número de especies, número de plantas por especie, cantidad de plantas en
  una hectárea, número de variables consideradas, número de parámetros.
- Experimentación: realizar 1000 simulaciones del proceso de siembra. Reportar:
  - valor esperado del total de plantas existentes en una hectárea;
  - valor esperado de plantas por especie existentes en una hectárea;
  - valor esperado de plantas por especie a suministrar por hectárea;
  - valor esperado de competencia existente en una hectárea;
  - probabilidad de transición entre plantas (considere una planta como un estado);
  - el orden en que se debe realizar la plantación (un ejemplo);
  - conclusiones y recomendaciones (las redacta el equipo; RESUMEN da los números).

## Decisiones del usuario (2026-10-20)

1. **1000 simulaciones.** El Monte Carlo ya usa 1000 escenarios en las 31 instancias. El paso 3
   (competencia con existentes) usa 1000 escenarios **sólo en 1 ha y P1 (map_27)**; las otras 29
   siguen con 100 y RESUMEN reporta la precisión (IC 95 % de la media).
2. **Transición:** se reportan **ambas** matrices 10×10 de especies:
   (a) por orden de siembra, P(siguiente planta sembrada es k | la actual es j);
   (b) por vecindad inmediata en la malla, P(un vecino inmediato es k | la planta es j).
3. **Orden de plantación:** un ejemplo para 1 ha (y P1) sobre el patrón del paso 3, ω = 0.
4. **DSATUR por cuotas (§10)** en las **31** instancias, como variantes de `reto practica`.
5. **Sensibilidades** κ ∈ {0.4, 0.8}, radios × {0.75, 1.25}, m × {0, 0.5, 2}, s2 (usar_s2):
   cadena completa (paso 1 γ*, Monte Carlo, paso 2, paso 3, práctica) en **1 ha y P1**, y
   **sólo paso 1** en **map_21** (8 ha) para comprobar que las conclusiones valen en un polígono
   grande.
6. **Validación CP-SAT:** la ajustada se vuelve a correr al final con 32 workers, 600 s,
   `--formulaciones ajustada --actualizar` (conserva las filas básicas de 8 workers).
7. Primero se implementa y prueba todo con corridas cortas; las corridas largas van al final.
   La re-corrida de la fase 4 (`run_all.ps1 -OmitirValidacion`) se hizo el 2026-10-20 y es la
   base de la fase 5; sus resultados no se reescriben salvo lo que cambia la fase 5 (paso 3 de
   1 ha y P1, práctica con DSATUR, salidas nuevas, RESUMEN y figuras).

## Ajustes del usuario al plan (2026-10-20, tras revisarlo)

1. Transiciones sobre los 1000 escenarios: `reto paso3` acumula, en 1 ha y map_27, los conteos
   10×10 por vecindad inmediata (incluye existentes) y por orden de siembra en serpentina (salta
   existentes y vacías) sobre todos los escenarios → `results/paso3_transicion_<inst>.csv`
   (de, a, conteo, n, prob). Esas son las matrices principales; la de ω = 0 queda como ejemplo.
   Prueba: la distribución estacionaria de la matriz por vecindad queda cerca de la composición
   plantada.
2. Plantas a suministrar: además de E[C_j] y q_j, E[n_j] = E[min(q_j, C_j)] por especie y por ha.
   IC 95 % (media ± 1.96·sd/√n) en E[T]/ha, E[X_j]/ha, E[C_j]/ha, E[n_j]/ha y E[Z].
3. Tiempos: `run_all.ps1` mide cada paso (Measure-Command) → `results/tiempos.csv`
   (paso, instancias, segundos, fecha); RESUMEN lo muestra junto a "Entorno y software".
4. entorno.py: tipo y velocidad de la RAM (Win32_PhysicalMemory: SMBIOSMemoryType, Speed,
   Capacity por módulo), tipo de disco (Get-PhysicalDisk: MediaType) y hash del commit.
5. tamano.py: (a) fila del modelo estocástico completo en forma extensiva (x, y replicadas por
   |Ω| = 1000 más las variables del paso 2) para justificar la descomposición; (b) fila TOTAL en
   `tamano_parametros.csv`; (c) D̄ además de D* en `tamano_problema.csv` (1 ha y map_27).
6. Antes de `reto paso3 --instancias todas` con 1000 escenarios, comprobar que
   `paso3_patron_<inst>_omega0.csv` y la fila ω = 0 de `paso3_escenarios.csv` salen idénticos al
   pasar de 100 a 1000 escenarios; si no, avisar al usuario antes de seguir.
7. (Opcional, aceptado) Z por nivel de competencia por escenario en `paso3_escenarios.csv`
   (columnas al final) para reportar E[Z por nivel]/ha.
