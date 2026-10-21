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
