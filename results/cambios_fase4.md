# Cambios de la fase 4

Antes: `be65ce0`. Después: `45c370f`.

## Cifras antes y después

| cifra | instancia | antes | despues | cambio_pct |
| --- | --- | --- | --- | --- |
| Z_opt paso 1 | 1ha | 577.3623 | 571.4363 | -1.0264 |
| D* paso 1 | 1ha | [46, 177, 38, 46, 53, 41, 80, 58, 94, 25] | [46, 178, 38, 46, 53, 41, 80, 58, 94, 24] |  |
| Z media paso 3 | 1ha | 571.0633 | 565.399 | -0.9919 |
| Z_opt paso 1 | map_27 | 746.4909 | 739.6761 | -0.9129 |
| D* paso 1 | map_27 | [59, 226, 49, 58, 69, 52, 102, 74, 121, 32] | [59, 226, 49, 59, 69, 52, 102, 74, 121, 31] |  |
| Z media paso 3 | map_27 | 743.3964 | 734.0388 | -1.2588 |
| Z_opt paso 1 | map_21 | 4970.0259 | 4855.1533 | -2.3113 |
| D* paso 1 | map_21 | [369, 1412, 303, 369, 431, 333, 642, 461, 756, 188] | [369, 1412, 303, 369, 431, 333, 642, 461, 756, 188] | 0 |
| Z media paso 3 | map_21 | 5061.0755 | 4938.289 | -2.4261 |
| costo paso 2 | 1ha | 12422 | 12422 | 0 |
| q paso 2 | 1ha | [38, 139, 30, 38, 43, 33, 65, 46, 78, 20] | [38, 140, 30, 38, 43, 33, 65, 46, 78, 19] |  |
| reducción vs serpentina iii | 1ha | 26.5125 | 27.4984 | 3.7188 |
| costo paso 2 | map_27 | 15892 | 15892 | 0 |
| q paso 2 | map_27 | [49, 177, 39, 48, 56, 42, 83, 58, 100, 26] | [49, 177, 39, 49, 56, 42, 83, 58, 100, 25] |  |
| reducción vs serpentina iii | map_27 | 26.9972 | 28.0504 | 3.901 |

## Validación exacta (filas definitivas)

| id | modelo | formulacion | z_opt | cota | optimo_probado | z_ts_mejor | z_ts_media | brecha_mejor | brecha_media |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| p1_a | paso1 | ajustada | 202.9292 | 187.5422 | False | 202.9399 | 203.461 | 0.0822 | 0.085 |
| p1_b | paso1 | ajustada | 270.1126 | 246.1276 | False | 270.1699 | 270.5203 | 0.0978 | 0.0992 |
| p1_c | paso1 | ajustada | 430.3323 | 384.1323 | False | 430.3823 | 431.738 | 0.1205 | 0.124 |
| p3_a | paso3 | ajustada | 34.9464 | 28.9393 | False | 34.9629 | 35.1711 | 0.2088 | 0.216 |
| p3_b | paso3 | ajustada | 60.2781 | 48.2767 | False | 60.3352 | 61.1635 | 0.2505 | 0.2676 |
| p3_c | paso3 | ajustada | 37.7158 | 35.9559 | False | 37.7193 | 37.7841 | 0.0494 | 0.0512 |
| p3_d | paso3 | ajustada | 49.2267 | 45.8829 | False | 49.2329 | 49.2756 | 0.0734 | 0.0743 |
| p3_e | paso3 | ajustada | 81.7072 | 74.9419 | False | 81.7573 | 81.8452 | 0.0913 | 0.0925 |
| p3_f | paso3 | ajustada | 40.5758 | 36.5227 | False | 40.5789 | 40.601 | 0.1115 | 0.1121 |
| p3_g | paso3 | ajustada | 62.1701 | 54.6096 | False | 62.2362 | 62.3842 | 0.1402 | 0.1429 |

## Parámetros del tabú

| candidatos | tenure | max_iter | max_sin_mejora | prob_intercambio | iter_por_posicion | sin_mejora_por_posicion |
| --- | --- | --- | --- | --- | --- | --- |
| 128 | 10 | 20000 | 3000 | 0.5 | 300 | 8 |
