# Resumen de resultados: reforestación CONAFOR (Entregable 2)

Generado por `python/resumen.py` a partir de los CSV y JSON de `results/`.

## Entorno y software

- Modelo de la PC: LENOVO 83DG
- Sistema operativo: Microsoft Windows 11 Home Single Language (versión 10.0.26300)
- Procesador: Intel(R) Core(TM) i9-14900HX (24 núcleos físicos, 32 hilos lógicos)
- Memoria RAM: 31.7 GB (2 × 16 GB DDR5 a 5600 MHz)
- Disco duro (C:): 826.4 GB totales, 238.5 GB libres, tipo SSD
- Commit del código: `5ac1a307cc4e13c3ed0b82ef7dd2fcdaf421acc5`

Tamaños en GiB (2³⁰ bytes), de `entorno.json` (`python/entorno.py`).

Software utilizado:

| software | versión |
| --- | --- |
| rustc | rustc 1.99.1 (58cf8f9c3 2026-10-13) |
| cargo | cargo 1.99.1 (5f94df478 2026-08-27) |
| python | 3.12.5 |
| numpy | 2.5.4 |
| scipy | 1.18.1 |
| pandas | 3.0.6 |
| matplotlib | 3.11.2 |
| pulp | 4.0.1 |
| highspy | 1.15.1 |
| ortools | 9.15.6755 |
| pytest | 9.1.1 |

Tiempos de cómputo de la última corrida de cada paso (`run_all.ps1` los mide con `Measure-Command` y añade una fila por paso a `results/tiempos.csv`, que conserva todas las corridas). Suma: 1,927.9 s (0.54 h).

| paso | instancias | segundos | fecha |
| --- | --- | --- | --- |
| cargo build --release --bin reto |  | 0.274 | 2026-10-21T23:56:08 |
| reto datos |  | 0.014 | 2026-10-21T23:56:08 |
| reto malla |  | 0.166 | 2026-10-21T23:56:08 |
| python python\ajuste.py |  | 1.819 | 2026-10-21T23:56:10 |
| reto paso1 | 1ha,map_27 | 17.564 | 2026-10-21T23:56:28 |
| reto paso1 --solo-gamma-star | todas | 385.921 | 2026-10-22T00:02:54 |
| reto montecarlo | todas | 0.604 | 2026-10-22T00:02:54 |
| python python\paso2_compra.py |  | 8.367 | 2026-10-22T00:03:03 |
| reto paso3 | todas | 1127.113 | 2026-10-22T00:21:50 |
| reto practica | todas | 1.549 | 2026-10-22T00:21:51 |
| python python\transicion.py |  | 0.709 | 2026-10-22T00:21:52 |
| python python\entorno.py |  | 5.708 | 2026-10-22T00:21:58 |
| python python\tamano.py |  | 0.931 | 2026-10-22T00:21:59 |
| python python\experimentacion.py |  | 0.704 | 2026-10-22T00:21:59 |
| python python\sensibilidad.py |  | 369.205 | 2026-10-22T00:28:09 |
| python python\practica_compra.py |  | 0.858 | 2026-10-22T00:28:09 |
| python python\markov.py |  | 0.722 | 2026-10-22T00:28:10 |
| python python\figuras.py |  | 5.636 | 2026-10-22T00:28:16 |

## Tamaño del problema

Datos del problema (1 ha y P1): número de especies, plantas por hectárea (D_tot/área), plantas por especie D*_j (paso 1, γ*) y D̄_j = `Dnom_j` (densidad nominal × área), y posiciones de la malla |V| (`n_pos`):

| instancia | especies | plantas_por_ha | D_tot | n_pos | D_0 | D_1 | D_2 | D_3 | D_4 | D_5 | D_6 | D_7 | D_8 | D_9 | Dnom_0 | Dnom_1 | Dnom_2 | Dnom_3 | Dnom_4 | Dnom_5 | Dnom_6 | Dnom_7 | Dnom_8 | Dnom_9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | 10 | 658 | 658 | 658 | 46 | 178 | 38 | 46 | 53 | 41 | 80 | 58 | 94 | 24 | 42 | 196 | 42 | 42 | 49 | 38 | 73 | 64 | 86 | 26 |
| map_27 | 10 | 657.8125 | 842 | 842 | 59 | 226 | 49 | 59 | 69 | 52 | 102 | 74 | 121 | 31 | 54 | 251 | 54 | 54 | 63 | 48 | 93 | 82 | 110 | 33 |

Variables y restricciones por modelo. La fila `extensivo_completo` es el modelo estocástico completo en forma extensiva (x, y del acomodo replicadas por cada uno de los |Ω| = 1000 escenarios más las variables del paso 2); su tamaño justifica resolverlo por pasos:

| instancia | modelo | variables | tipo_variables | restricciones | nota |
| --- | --- | --- | --- | --- | --- |
| 1ha | paso1_tabu | 658 | enteras en {0..10} | 30 | una variable por posición; cuotas L_j <= n_j <= U_j y composición como penalización/filtro de la búsqueda |
| 1ha | paso1_lineal | 550880 | binarias | 1633588 | 10*\|V\| binarias x + 100*\|E\| y, \|V\|=658, \|E\|=5443; restricciones \|V\|+10+20+3*100*\|E\| |
| 1ha | paso2_ple | 436 | 10 enteras q + 426 continuas f/e | 427 | conteos reales del modelo agrupado por valores distintos de C_j; la forma completa sin agrupar tiene 10 q + 2*10*\|Omega\| f/e (\|Omega\|=1000 en esta corrida) |
| 1ha | paso3_por_escenario | 526 | enteras en {0..10} | 10 | posiciones libres \|V_lib\| del escenario omega=0; se resuelve una vez por cada uno de los \|Omega\|=1000 |
| 1ha | extensivo_completo | 551450890 | binarias y enteras | 1635221589 | x, y del paso 1 + q (10) + por cada omega de \|Omega\|=1000: x_w, y_w del acomodo (10*\|V\|+100*\|E\|) y f_w, e_w (20); justifica la descomposición en pasos |
| map_27 | paso1_tabu | 842 | enteras en {0..10} | 30 | una variable por posición; cuotas L_j <= n_j <= U_j y composición como penalización/filtro de la búsqueda |
| map_27 | paso1_lineal | 711620 | binarias | 2110472 | 10*\|V\| binarias x + 100*\|E\| y, \|V\|=842, \|E\|=7032; restricciones \|V\|+10+20+3*100*\|E\| |
| map_27 | paso2_ple | 478 | 10 enteras q + 468 continuas f/e | 469 | conteos reales del modelo agrupado por valores distintos de C_j; la forma completa sin agrupar tiene 10 q + 2*10*\|Omega\| f/e (\|Omega\|=1000 en esta corrida) |
| map_27 | paso3_por_escenario | 670 | enteras en {0..10} | 10 | posiciones libres \|V_lib\| del escenario omega=0; se resuelve una vez por cada uno de los \|Omega\|=1000 |
| map_27 | extensivo_completo | 712351630 | binarias y enteras | 2112582473 | x, y del paso 1 + q (10) + por cada omega de \|Omega\|=1000: x_w, y_w del acomodo (10*\|V\|+100*\|E\|) y f_w, e_w (20); justifica la descomposición en pasos |

Parámetros (la fila TOTAL es la suma de la columna `cantidad`):

| parametro | simbolo | cantidad | fuente |
| --- | --- | --- | --- |
| costo unitario | c_j | 10 | data/especies.csv |
| supervivencia a 1 año | s1_j | 10 | data/supervivencia.csv |
| supervivencia a 2 años | s2_j | 10 | data/supervivencia.csv |
| densidad nominal | δ_j | 10 | data/especies.csv |
| radio de copa | R_j | 10 | data/rasgos.csv (radio_d) |
| rasgos (7 por especie) | rasgos | 70 | data/rasgos.csv |
| matriz de interacción | W | 100 | construida de rasgos |
| umbral de poda | θ_j | 10 | derivado de c, s1, W (config m_escala) |
| cota inferior de composición | L_j | 10 | paso 1 (ε) |
| cota superior de composición | U_j | 10 | paso 1 (ε) |
| penalización por faltante | π_j | 10 | pi_factor * c_j |
| distancia de vecindad | d | 1 | config/default.json |
| tolerancia de composición | ε | 1 | config/default.json |
| factor de vecindad | κ | 1 | config/default.json |
| peso de interacción elegido | γ* | 1 | results/paso1_gamma_star.csv |
| factor de radios | factor_radios | 1 | config/default.json |
| presupuesto por ha | B | 1 | config/default.json |
| penalización por sobrante | η | 1 | config (0 por defecto) |
| media de plantas Monte Carlo | μ_ρ | 1 | results/mc_params.json |
| desviación de plantas Monte Carlo | σ_ρ | 1 | results/mc_params.json |
| proporciones de especie | p̂_j | 10 | results/mc_params.json |
| escenarios Monte Carlo | \|Ω\| | 1 | config/default.json |
| escenarios del paso 3 | \|Ω₃\| | 1 | config/default.json |
| TOTAL |  | 281 | suma |

## Malla

### 1 ha y P1 (map_27)

| instancia | area_ha | \|V\| | round(658·A) | excedente | d_tot |
| --- | --- | --- | --- | --- | --- |
| 1ha | 1 | 658 | 658 | 0 | 658 |
| map_27 | 1.28 | 842 | 842 | 0 | 842 |

Pares por anillo:

| anillo | 1ha | map_27 |
| --- | --- | --- |
| 1 | 1872 | 2409 |
| 2 | 1799 | 2327 |
| 3 | 1772 | 2296 |

### Las 31 instancias

| instancia | area_ha | columnas | filas | \|V\| | round(658·A) | excedente | d_tot | pares_anillo_1 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | 1 | 24 | 28 | 658 | 658 | 0 | 658 | 1872 |
| map_27 | 1.28 | 26 | 33 | 842 | 842 | 0 | 842 | 2409 |
| map_28 | 6.64 | 58 | 76 | 4370 | 4369 | 1 | 4369 | 12844 |
| map_30 | 6.76 | 65 | 69 | 4451 | 4448 | 3 | 4448 | 13086 |
| map_20 | 1.38 | 28 | 33 | 908 | 908 | 0 | 908 | 2603 |
| map_21 | 8 | 61 | 87 | 5264 | 5264 | 0 | 5264 | 15497 |
| map_22 | 7.82 | 62 | 83 | 5146 | 5146 | 0 | 5146 | 15149 |
| map_23 | 5.53 | 56 | 65 | 3640 | 3639 | 1 | 3639 | 10679 |
| map_24 | 5.64 | 58 | 64 | 3712 | 3711 | 1 | 3711 | 10893 |
| map_18 | 7.11 | 65 | 72 | 4680 | 4678 | 2 | 4678 | 13767 |
| map_17 | 6.11 | 60 | 67 | 4020 | 4020 | 0 | 4020 | 11807 |
| map_16 | 5.64 | 58 | 64 | 3712 | 3711 | 1 | 3711 | 10893 |
| map_19 | 4.92 | 54 | 60 | 3240 | 3237 | 3 | 3237 | 9493 |
| map_25 | 5.05 | 55 | 61 | 3325 | 3323 | 2 | 3323 | 9744 |
| map_26 | 4.75 | 53 | 59 | 3127 | 3126 | 1 | 3126 | 9158 |
| map_13 | 7.97 | 69 | 76 | 5244 | 5244 | 0 | 5244 | 15443 |
| map_31 | 7.34 | 69 | 70 | 4830 | 4830 | 0 | 4830 | 14213 |
| map_14 | 5.98 | 65 | 61 | 3935 | 3935 | 0 | 3935 | 11554 |
| map_15 | 5.4 | 52 | 69 | 3554 | 3553 | 1 | 3553 | 10421 |
| map_7 | 6.28 | 60 | 69 | 4140 | 4132 | 8 | 4132 | 12163 |
| map_8 | 7.6 | 69 | 73 | 5001 | 5001 | 0 | 5001 | 14720 |
| map_9 | 8 | 61 | 87 | 5264 | 5264 | 0 | 5264 | 15497 |
| map_10 | 8 | 61 | 87 | 5264 | 5264 | 0 | 5264 | 15497 |
| map_11 | 7.67 | 67 | 76 | 5054 | 5047 | 7 | 5047 | 14878 |
| map_12 | 1.47 | 29 | 34 | 969 | 967 | 2 | 967 | 2783 |
| map_6 | 4.19 | 45 | 62 | 2759 | 2757 | 2 | 2757 | 8065 |
| map_2 | 7.52 | 66 | 75 | 4950 | 4948 | 2 | 4948 | 14569 |
| map_3 | 8 | 61 | 87 | 5264 | 5264 | 0 | 5264 | 15497 |
| map_4 | 8 | 61 | 87 | 5264 | 5264 | 0 | 5264 | 15497 |
| map_5 | 7.56 | 63 | 79 | 4977 | 4974 | 3 | 4974 | 14648 |
| map_1 | 5.4 | 52 | 69 | 3554 | 3553 | 1 | 3553 | 10421 |

## Ajuste de distribuciones

| estadistico | valor |
| --- | --- |
| n_poligonos | 30 |
| media_rho | 129.1823 |
| sd_rho | 3.3458 |
| sigma_mle | 3.2896 |
| shapiro_p | 0.2052 |
| dispersion_poisson | 0.4103 |
| corr_rho_area | -0.3788 |
| chi2_homog | 286.6504 |
| chi2_gl | 261 |
| chi2_p | 0.132 |
| precision_dm | 15276.9561 |
| loglik_dm | -49777.7847 |
| loglik_multinomial | -49777.9728 |

Ajuste de la densidad ρ:

| modelo | parametros | loglik | AIC | KS_p |
| --- | --- | --- | --- | --- |
| Normal | 129.182340 3.289563 | -78.2908 | 160.5816 | 0.6403 |
| Gamma | 1550.839238 0.000000 0.083298 | -78.2002 | 160.4004 | 0.6103 |
| Lognormal | 0.025363 0.000000 129.140693 | -78.1613 | 160.3227 | 0.5953 |

Parámetros de Monte Carlo (`mc_params.json`): μ = 129.1823, σ = 3.2896.

| especie | p_hat |
| --- | --- |
| 0 | 0.0633 |
| 1 | 0.2951 |
| 2 | 0.062 |
| 3 | 0.0596 |
| 4 | 0.0777 |
| 5 | 0.0619 |
| 6 | 0.1165 |
| 7 | 0.0971 |
| 8 | 0.128 |
| 9 | 0.0387 |

## Paso 1: composición y competencia

γ* = 3, elegido por el usuario en el codo de la curva de compromiso.

Barrido de γ, 1ha:

| gamma | suma_theta_d | suma_c_d | z_opt_media | z_opt_min | z_opt_max | z_opt_sd | z_opt_por_planta_media |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 1274.0922 | 15500 | 614.6018 | 610.0119 | 618.9356 | 4.4674 | 0.934 |
| 0.1 | 1274.0922 | 15500 | 607.3206 | 607.0669 | 607.612 | 0.2745 | 0.923 |
| 0.3 | 1274.6018 | 15452 | 607.5857 | 606.2019 | 610.1683 | 2.2385 | 0.9234 |
| 1 | 1277.3292 | 15444 | 599.9329 | 597.9214 | 601.2848 | 1.7761 | 0.9118 |
| 3 | 1343.7892 | 15422 | 571.4363 | 569.0427 | 574.7853 | 2.9881 | 0.8684 |
| 10 | 1350.9808 | 15422 | 571.6676 | 571.2321 | 572.4726 | 0.6979 | 0.8688 |
| 30 | 1350.9808 | 15422 | 572.9945 | 572.3065 | 573.3735 | 0.5969 | 0.8708 |
| 100 | 1350.9808 | 15422 | 573.6912 | 571.661 | 576.7734 | 2.7137 | 0.8719 |

Barrido de γ, map_27:

| gamma | suma_theta_d | suma_c_d | z_opt_media | z_opt_min | z_opt_max | z_opt_sd | z_opt_por_planta_media |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 1626.8441 | 19833.5 | 789.4162 | 785.0944 | 792.8232 | 3.9448 | 0.9375 |
| 0.1 | 1626.8441 | 19833.5 | 786.0423 | 781.3402 | 788.7523 | 4.0879 | 0.9335 |
| 0.3 | 1627.3355 | 19761.5 | 784.2126 | 781.3567 | 788.7544 | 3.9766 | 0.9314 |
| 1 | 1632.5303 | 19761.5 | 777.0214 | 774.108 | 780.2459 | 3.0808 | 0.9228 |
| 3 | 1723.8049 | 19732.5 | 739.6761 | 738.8838 | 740.3301 | 0.733 | 0.8785 |
| 10 | 1732.5669 | 19732.5 | 736.4782 | 735.3834 | 738.5877 | 1.8273 | 0.8747 |
| 30 | 1732.5669 | 19732.5 | 737.177 | 735.8773 | 738.3533 | 1.2426 | 0.8755 |
| 100 | 1732.5669 | 19732.5 | 738.5592 | 738.3483 | 738.9622 | 0.3492 | 0.8771 |

D* por instancia (γ*):

| instancia | d_tot | D_0 | D_1 | D_2 | D_3 | D_4 | D_5 | D_6 | D_7 | D_8 | D_9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | 658 | 46 | 178 | 38 | 46 | 53 | 41 | 80 | 58 | 94 | 24 |
| map_27 | 842 | 59 | 226 | 49 | 59 | 69 | 52 | 102 | 74 | 121 | 31 |
| map_28 | 4369 | 306 | 1171 | 252 | 306 | 357 | 276 | 533 | 383 | 628 | 157 |
| map_30 | 4448 | 312 | 1193 | 256 | 312 | 364 | 280 | 542 | 390 | 639 | 160 |
| map_20 | 908 | 63 | 243 | 53 | 63 | 74 | 57 | 111 | 80 | 130 | 34 |
| map_21 | 5264 | 369 | 1412 | 303 | 369 | 431 | 333 | 642 | 461 | 756 | 188 |
| map_22 | 5146 | 361 | 1380 | 296 | 360 | 421 | 326 | 628 | 451 | 740 | 183 |
| map_23 | 3639 | 255 | 976 | 209 | 254 | 298 | 231 | 444 | 319 | 523 | 130 |
| map_24 | 3711 | 260 | 995 | 214 | 260 | 303 | 235 | 453 | 325 | 533 | 133 |
| map_18 | 4678 | 328 | 1254 | 270 | 328 | 382 | 297 | 570 | 410 | 672 | 167 |
| map_17 | 4020 | 282 | 1078 | 232 | 282 | 328 | 254 | 490 | 352 | 577 | 145 |
| map_16 | 3711 | 260 | 995 | 214 | 260 | 303 | 235 | 453 | 325 | 533 | 133 |
| map_19 | 3237 | 227 | 868 | 187 | 226 | 265 | 205 | 394 | 284 | 465 | 116 |
| map_25 | 3323 | 233 | 891 | 191 | 233 | 272 | 211 | 405 | 291 | 477 | 119 |
| map_26 | 3126 | 220 | 839 | 180 | 218 | 256 | 198 | 381 | 274 | 449 | 111 |
| map_13 | 5244 | 368 | 1406 | 302 | 368 | 429 | 331 | 640 | 459 | 753 | 188 |
| map_31 | 4830 | 338 | 1296 | 278 | 338 | 396 | 306 | 589 | 423 | 694 | 172 |
| map_14 | 3935 | 276 | 1055 | 226 | 276 | 322 | 247 | 480 | 345 | 565 | 143 |
| map_15 | 3553 | 249 | 953 | 205 | 249 | 291 | 225 | 433 | 312 | 510 | 126 |
| map_7 | 4132 | 290 | 1108 | 238 | 290 | 338 | 260 | 503 | 362 | 594 | 149 |
| map_8 | 5001 | 350 | 1341 | 288 | 350 | 409 | 317 | 610 | 438 | 719 | 179 |
| map_9 | 5264 | 369 | 1412 | 303 | 369 | 431 | 333 | 642 | 461 | 756 | 188 |
| map_10 | 5264 | 369 | 1412 | 303 | 369 | 431 | 333 | 642 | 461 | 756 | 188 |
| map_11 | 5047 | 354 | 1353 | 290 | 354 | 413 | 319 | 616 | 442 | 726 | 180 |
| map_12 | 967 | 68 | 260 | 56 | 67 | 79 | 58 | 117 | 85 | 136 | 41 |
| map_6 | 2757 | 193 | 739 | 159 | 193 | 225 | 174 | 336 | 242 | 397 | 99 |
| map_2 | 4948 | 347 | 1327 | 285 | 347 | 404 | 314 | 603 | 433 | 711 | 177 |
| map_3 | 5264 | 369 | 1412 | 303 | 369 | 431 | 333 | 642 | 461 | 756 | 188 |
| map_4 | 5264 | 369 | 1412 | 303 | 369 | 431 | 333 | 642 | 461 | 756 | 188 |
| map_5 | 4974 | 349 | 1334 | 286 | 347 | 407 | 314 | 607 | 436 | 715 | 179 |
| map_1 | 3553 | 249 | 953 | 205 | 249 | 291 | 225 | 433 | 312 | 510 | 126 |

## Monte Carlo

Por especie (1 ha y P1):

| instancia | composicion | especie | D | E_C | sd_C | P90 | P95 | N_necesario |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | nominal | 0 | 42 | 33.761 | 2.8478 | 37 | 38 | 125 |
| 1ha | nominal | 1 | 196 | 157.784 | 5.3086 | 164 | 167 | 434 |
| 1ha | nominal | 2 | 42 | 34.062 | 2.7914 | 38 | 38 | 120 |
| 1ha | nominal | 3 | 42 | 34.282 | 2.6056 | 38 | 38 | 105 |
| 1ha | nominal | 4 | 49 | 38.813 | 3.1041 | 43 | 44 | 149 |
| 1ha | nominal | 5 | 38 | 29.909 | 2.7394 | 33 | 34 | 116 |
| 1ha | nominal | 6 | 73 | 57.926 | 3.7262 | 63 | 64 | 214 |
| 1ha | nominal | 7 | 64 | 51.496 | 3.3988 | 56 | 57 | 178 |
| 1ha | nominal | 8 | 86 | 69.606 | 3.8597 | 74.1 | 76 | 229 |
| 1ha | nominal | 9 | 26 | 20.894 | 2.2592 | 24 | 24 | 79 |
| 1ha | gamma_star | 0 | 46 | 37.761 | 2.8478 | 41 | 42 | 125 |
| 1ha | gamma_star | 1 | 178 | 139.784 | 5.3086 | 146 | 149 | 434 |
| 1ha | gamma_star | 2 | 38 | 30.062 | 2.7914 | 34 | 34 | 120 |
| 1ha | gamma_star | 3 | 46 | 38.282 | 2.6056 | 42 | 42 | 105 |
| 1ha | gamma_star | 4 | 53 | 42.813 | 3.1041 | 47 | 48 | 149 |
| 1ha | gamma_star | 5 | 41 | 32.909 | 2.7394 | 36 | 37 | 116 |
| 1ha | gamma_star | 6 | 80 | 64.926 | 3.7262 | 70 | 71 | 214 |
| 1ha | gamma_star | 7 | 58 | 45.496 | 3.3988 | 50 | 51 | 178 |
| 1ha | gamma_star | 8 | 94 | 77.606 | 3.8597 | 82.1 | 84 | 229 |
| 1ha | gamma_star | 9 | 24 | 18.894 | 2.2592 | 22 | 22 | 79 |
| map_27 | nominal | 0 | 54 | 43.544 | 3.215 | 47 | 49 | 159 |
| map_27 | nominal | 1 | 251 | 202.265 | 5.8649 | 209 | 211 | 529 |
| map_27 | nominal | 2 | 54 | 43.911 | 3.1946 | 48 | 49 | 157 |
| map_27 | nominal | 3 | 54 | 44.055 | 3.0626 | 48 | 49 | 145 |
| map_27 | nominal | 4 | 63 | 50.033 | 3.5177 | 55 | 56 | 191 |
| map_27 | nominal | 5 | 48 | 37.518 | 3.1086 | 42 | 43 | 149 |
| map_27 | nominal | 6 | 93 | 73.856 | 4.1798 | 79 | 80 | 269 |
| map_27 | nominal | 7 | 82 | 65.826 | 3.944 | 71 | 72 | 240 |
| map_27 | nominal | 8 | 110 | 89.036 | 4.1745 | 94 | 96 | 268 |
| map_27 | nominal | 9 | 33 | 26.624 | 2.4529 | 30 | 30 | 93 |
| map_27 | gamma_star | 0 | 59 | 48.544 | 3.215 | 52 | 54 | 159 |
| map_27 | gamma_star | 1 | 226 | 177.265 | 5.8649 | 184 | 186 | 529 |
| map_27 | gamma_star | 2 | 49 | 38.911 | 3.1946 | 43 | 44 | 157 |
| map_27 | gamma_star | 3 | 59 | 49.055 | 3.0626 | 53 | 54 | 145 |
| map_27 | gamma_star | 4 | 69 | 56.033 | 3.5177 | 61 | 62 | 191 |
| map_27 | gamma_star | 5 | 52 | 41.518 | 3.1086 | 46 | 47 | 149 |
| map_27 | gamma_star | 6 | 102 | 82.856 | 4.1798 | 88 | 89 | 269 |
| map_27 | gamma_star | 7 | 74 | 57.826 | 3.944 | 63 | 64 | 240 |
| map_27 | gamma_star | 8 | 121 | 100.036 | 4.1745 | 105 | 107 | 268 |
| map_27 | gamma_star | 9 | 31 | 24.624 | 2.4529 | 28 | 28 | 93 |

Costo (las 31 instancias):

| instancia | composicion | costo_esperado | costo_sd | presupuesto | P_costo_mayor_B | violaciones_X_mayor_L | escenarios_C_negativo | especies_C_negativo |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | nominal | 12441.236 | 89.2842 | 12500 | 0.265 | 0 | 0 | 0 |
| 1ha | gamma_star | 12390.236 | 89.2842 | 12500 | 0.111 | 0 | 0 | 0 |
| map_27 | nominal | 15932.576 | 106.5177 | 16000 | 0.27 | 0 | 0 | 0 |
| map_27 | gamma_star | 15862.076 | 106.5177 | 16000 | 0.1 | 0 | 0 | 0 |
| map_28 | nominal | 82633.866 | 536.6534 | 83000 | 0.254 | 0 | 0 | 0 |
| map_28 | gamma_star | 82238.366 | 536.6534 | 83000 | 0.072 | 0 | 0 | 0 |
| map_30 | nominal | 84146.852 | 525.3404 | 84500 | 0.263 | 0 | 0 | 0 |
| map_30 | gamma_star | 83748.852 | 525.3404 | 84500 | 0.076 | 0 | 0 | 0 |
| map_20 | nominal | 17178.1945 | 114.6888 | 17250 | 0.263 | 0 | 0 | 0 |
| map_20 | gamma_star | 17094.6945 | 114.6888 | 17250 | 0.074 | 0 | 0 | 0 |
| map_21 | nominal | 99588.404 | 606.1036 | 100000 | 0.241 | 0 | 0 | 0 |
| map_21 | gamma_star | 99109.404 | 606.1036 | 100000 | 0.079 | 0 | 0 | 0 |
| map_22 | nominal | 97308.012 | 600.8614 | 97750 | 0.227 | 0 | 0 | 0 |
| map_22 | gamma_star | 96836.512 | 600.8614 | 97750 | 0.06 | 0 | 0 | 0 |
| map_23 | nominal | 68826.469 | 444.3793 | 69125 | 0.251 | 0 | 0 | 0 |
| map_23 | gamma_star | 68488.969 | 444.3793 | 69125 | 0.088 | 0 | 0 | 0 |
| map_24 | nominal | 70183.27 | 451.8197 | 70500 | 0.24 | 0 | 0 | 0 |
| map_24 | gamma_star | 69846.27 | 451.8197 | 70500 | 0.074 | 0 | 0 | 0 |
| map_18 | nominal | 88484.532 | 543.1547 | 88875 | 0.242 | 0 | 0 | 0 |
| map_18 | gamma_star | 88060.032 | 543.1547 | 88875 | 0.063 | 0 | 0 | 0 |
| map_17 | nominal | 76071.854 | 463.553 | 76375 | 0.268 | 0 | 0 | 0 |
| map_17 | gamma_star | 75715.854 | 463.553 | 76375 | 0.078 | 0 | 0 | 0 |
| map_16 | nominal | 70196.4355 | 448.9079 | 70500 | 0.248 | 0 | 0 | 0 |
| map_16 | gamma_star | 69859.4355 | 448.9079 | 70500 | 0.073 | 0 | 0 | 0 |
| map_19 | nominal | 61249.424 | 404.0765 | 61500 | 0.255 | 0 | 0 | 0 |
| map_19 | gamma_star | 60956.424 | 404.0765 | 61500 | 0.1 | 0 | 0 | 0 |
| map_25 | nominal | 62844.4895 | 416.4351 | 63125 | 0.254 | 0 | 0 | 0 |
| map_25 | gamma_star | 62546.9895 | 416.4351 | 63125 | 0.074 | 0 | 0 | 0 |
| map_26 | nominal | 59131.8145 | 359.2813 | 59375 | 0.252 | 0 | 0 | 0 |
| map_26 | gamma_star | 58845.8145 | 359.2813 | 59375 | 0.065 | 0 | 0 | 0 |
| map_13 | nominal | 99186.159 | 620.3223 | 99625 | 0.239 | 0 | 0 | 0 |
| map_13 | gamma_star | 98712.159 | 620.3223 | 99625 | 0.071 | 0 | 0 | 0 |
| map_31 | nominal | 91369.438 | 579.0157 | 91750 | 0.26 | 0 | 0 | 0 |
| map_31 | gamma_star | 90929.938 | 579.0157 | 91750 | 0.084 | 0 | 0 | 0 |
| map_14 | nominal | 74425.0265 | 472.7624 | 74750 | 0.256 | 0 | 0 | 0 |
| map_14 | gamma_star | 74078.5265 | 472.7624 | 74750 | 0.078 | 0 | 0 | 0 |
| map_15 | nominal | 67233.692 | 436.6264 | 67500 | 0.262 | 0 | 0 | 0 |
| map_15 | gamma_star | 66908.692 | 436.6264 | 67500 | 0.089 | 0 | 0 | 0 |
| map_7 | nominal | 78133.1855 | 499.8837 | 78500 | 0.237 | 0 | 0 | 0 |
| map_7 | gamma_star | 77770.1855 | 499.8837 | 78500 | 0.07 | 0 | 0 | 0 |
| map_8 | nominal | 94576.489 | 606.6017 | 95000 | 0.234 | 0 | 0 | 0 |
| map_8 | gamma_star | 94116.989 | 606.6017 | 95000 | 0.077 | 0 | 0 | 0 |
| map_9 | nominal | 99568.6185 | 614.1332 | 100000 | 0.229 | 0 | 0 | 0 |
| map_9 | gamma_star | 99089.6185 | 614.1332 | 100000 | 0.071 | 0 | 0 | 0 |
| map_10 | nominal | 99579.922 | 621.5833 | 100000 | 0.252 | 0 | 0 | 0 |
| map_10 | gamma_star | 99100.922 | 621.5833 | 100000 | 0.065 | 0 | 0 | 0 |
| map_11 | nominal | 95453.5305 | 581.5102 | 95875 | 0.228 | 0 | 0 | 0 |
| map_11 | gamma_star | 94999.5305 | 581.5102 | 95875 | 0.063 | 0 | 0 | 0 |
| map_12 | nominal | 18293.168 | 127.8202 | 18375 | 0.252 | 0 | 0 | 0 |
| map_12 | gamma_star | 18224.168 | 127.8202 | 18375 | 0.115 | 0 | 0 | 0 |
| map_6 | nominal | 52161.098 | 333.7546 | 52375 | 0.26 | 0 | 0 | 0 |
| map_6 | gamma_star | 51912.098 | 333.7546 | 52375 | 0.082 | 0 | 0 | 0 |
| map_2 | nominal | 93585.636 | 625.7868 | 94000 | 0.244 | 0 | 0 | 0 |
| map_2 | gamma_star | 93141.636 | 625.7868 | 94000 | 0.084 | 0 | 0 | 0 |
| map_3 | nominal | 99561.072 | 629.1213 | 100000 | 0.257 | 0 | 0 | 0 |
| map_3 | gamma_star | 99082.072 | 629.1213 | 100000 | 0.078 | 0 | 0 | 0 |
| map_4 | nominal | 99562.736 | 632.9747 | 100000 | 0.252 | 0 | 0 | 0 |
| map_4 | gamma_star | 99083.736 | 632.9747 | 100000 | 0.074 | 0 | 0 | 0 |
| map_5 | nominal | 94104.8775 | 596.6588 | 94500 | 0.253 | 0 | 0 | 0 |
| map_5 | gamma_star | 93650.3775 | 596.6588 | 94500 | 0.083 | 0 | 0 | 0 |
| map_1 | nominal | 67217.1205 | 422.6849 | 67500 | 0.253 | 0 | 0 | 0 |
| map_1 | gamma_star | 66892.1205 | 422.6849 | 67500 | 0.074 | 0 | 0 | 0 |

## Paso 2: compra

ε = 0.1, π = 2·c (31 instancias):

| instancia | q_total | costo | presupuesto | cabe | faltante_esperado | sobrante_esperado |
| --- | --- | --- | --- | --- | --- | --- |
| 1ha | 530 | 12422 | 12500 | True | 12.264 | 13.731 |
| map_27 | 678 | 15892 | 16000 | True | 13.805 | 15.137 |
| map_28 | 3512 | 82280.5 | 83000 | True | 33.384 | 34.774 |
| map_30 | 3578 | 83818 | 84500 | True | 33.06 | 36.239 |
| map_20 | 730 | 17099.5 | 17250 | True | 14.868 | 15.161 |
| map_21 | 4232 | 99127 | 100000 | True | 37.417 | 38.259 |
| map_22 | 4135 | 96852 | 97750 | True | 37.348 | 38.036 |
| map_23 | 2925 | 68498.5 | 69125 | True | 30.536 | 31.308 |
| map_24 | 2983 | 69884 | 70500 | True | 31.07 | 32.326 |
| map_18 | 3761 | 88098 | 88875 | True | 34.831 | 36.631 |
| map_17 | 3232 | 75717 | 76375 | True | 32.368 | 32.576 |
| map_16 | 2982 | 69857.5 | 70500 | True | 31.498 | 31.311 |
| map_19 | 2605 | 61019.5 | 61500 | True | 27.546 | 30.201 |
| map_25 | 2672 | 62590 | 63125 | True | 28.924 | 30.411 |
| map_26 | 2512 | 58841.5 | 59375 | True | 28.604 | 28.443 |
| map_13 | 4216 | 98759.5 | 99625 | True | 36.695 | 38.535 |
| map_31 | 3882 | 90927 | 91750 | True | 36.7 | 36.453 |
| map_14 | 3164 | 74126 | 74750 | True | 31.096 | 33.124 |
| map_15 | 2858 | 66936.5 | 67500 | True | 30.22 | 31.508 |
| map_7 | 3321 | 77813 | 78500 | True | 32.285 | 33.854 |
| map_8 | 4020 | 94168.5 | 95000 | True | 35.88 | 37.745 |
| map_9 | 4232 | 99118.5 | 100000 | True | 36.972 | 38.545 |
| map_10 | 4232 | 99130 | 100000 | True | 37.312 | 38.825 |
| map_11 | 4058 | 95062 | 95875 | True | 35.686 | 38.334 |
| map_12 | 778 | 18246 | 18375 | True | 15.323 | 16.163 |
| map_6 | 2217 | 51946 | 52375 | True | 26.418 | 27.525 |
| map_2 | 3979 | 93221.5 | 94000 | True | 35.255 | 38.425 |
| map_3 | 4234 | 99182 | 100000 | True | 35.978 | 40.201 |
| map_4 | 4233 | 99153 | 100000 | True | 36.682 | 39.669 |
| map_5 | 3999 | 93671 | 94500 | True | 36.87 | 37.795 |
| map_1 | 2856 | 66892.5 | 67500 | True | 30.807 | 30.631 |

Comparación de π (1 ha y P1):

| instancia | pi_factor | q_total | costo | presupuesto | cabe | faltante_esperado | sobrante_esperado |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | 2 | 530 | 12422 | 12500 | True | 12.264 | 13.731 |
| 1ha | 4 | 533 | 12500 | 12500 | True | 10.85 | 15.317 |
| 1ha | 10 | 533 | 12500 | 12500 | True | 10.85 | 15.317 |
| map_27 | 2 | 678 | 15892 | 16000 | True | 13.805 | 15.137 |
| map_27 | 4 | 683 | 15998.5 | 16000 | True | 11.47 | 17.802 |
| map_27 | 10 | 683 | 15998.5 | 16000 | True | 11.47 | 17.802 |

## Paso 3: ajuste al polígono real

Los escenarios caen en `presupuesto_corto` en 4900 de 4900 casos. El nombre es el de la especificación para Σn < |V_lib|; aquí no lo causa el presupuesto. Con π = 2c la razón crítica es 0.5, así que el modelo compra cerca de la mediana de C, y además la malla tiene posiciones de más (|V| > D_tot). Como el caso `usual` exige Σn = |V_lib|, no se alcanza si algún q_j < C_j o si |V| > D_tot.

Huecos medios por hectárea, (V_lib − M)/A: entre 4.4 y 12.3 en las 31 instancias (12.3 en 1 ha, 10.8 en P1/map_27). Cada hueco se parte en faltante de compra (ΣC⁺ − M, que coincide con el `faltante_esperado` del paso 2 salvo por el número de escenarios: 100 aquí, 1000 allá) y excedente de malla (|V| − D_tot). El excedente de malla va de 0 a 8 posiciones.

El presupuesto no es activo con π = 2: la compra cabe en todas las instancias, con una holgura B·A − costo de $78 a $898. Es un hallazgo del modelo, no un error.

Huecos de paso 3 (1 ha y P1):

| instancia | area_ha | huecos | huecos_por_ha | faltante_compra | faltante_esperado_paso2 | excedente_malla | holgura_presupuesto |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | 1 | 12.264 | 12.264 | 12.264 | 12.264 | 0 | 78 |
| map_27 | 1.28 | 13.805 | 10.7852 | 13.805 | 13.805 | 0 | 108 |

Huecos de paso 3 (todas las instancias):

| instancia | area_ha | huecos | huecos_por_ha | faltante_compra | faltante_esperado_paso2 | excedente_malla | holgura_presupuesto |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | 1 | 12.264 | 12.264 | 12.264 | 12.264 | 0 | 78 |
| map_27 | 1.28 | 13.805 | 10.7852 | 13.805 | 13.805 | 0 | 108 |
| map_28 | 6.64 | 32.69 | 4.9232 | 31.69 | 33.384 | 1 | 719.5 |
| map_30 | 6.76 | 36.15 | 5.3476 | 33.15 | 33.06 | 3 | 682 |
| map_20 | 1.38 | 14.66 | 10.6232 | 14.66 | 14.868 | 0 | 150.5 |
| map_21 | 8 | 35.29 | 4.4112 | 35.29 | 37.417 | 0 | 873 |
| map_22 | 7.82 | 35.9 | 4.5908 | 35.9 | 37.348 | 0 | 898 |
| map_23 | 5.53 | 32.48 | 5.8734 | 31.48 | 30.536 | 1 | 626.5 |
| map_24 | 5.64 | 33.39 | 5.9202 | 32.39 | 31.07 | 1 | 616 |
| map_18 | 7.11 | 35.52 | 4.9958 | 33.52 | 34.831 | 2 | 777 |
| map_17 | 6.11 | 31.7 | 5.1882 | 31.7 | 32.368 | 0 | 658 |
| map_16 | 5.64 | 33.1 | 5.8688 | 32.1 | 31.498 | 1 | 642.5 |
| map_19 | 4.92 | 29.4 | 5.9756 | 26.4 | 27.546 | 3 | 480.5 |
| map_25 | 5.05 | 30.11 | 5.9624 | 28.11 | 28.924 | 2 | 535 |
| map_26 | 4.75 | 31.29 | 6.5874 | 30.29 | 28.604 | 1 | 533.5 |
| map_13 | 7.97 | 39.99 | 5.0176 | 39.99 | 36.695 | 0 | 865.5 |
| map_31 | 7.34 | 36.52 | 4.9755 | 36.52 | 36.7 | 0 | 823 |
| map_14 | 5.98 | 34.03 | 5.6906 | 34.03 | 31.096 | 0 | 624 |
| map_15 | 5.4 | 30.93 | 5.7278 | 29.93 | 30.22 | 1 | 563.5 |
| map_7 | 6.28 | 41.57 | 6.6194 | 33.57 | 32.285 | 8 | 687 |
| map_8 | 7.6 | 37.65 | 4.9539 | 37.65 | 35.88 | 0 | 831.5 |
| map_9 | 8 | 39.55 | 4.9437 | 39.55 | 36.972 | 0 | 881.5 |
| map_10 | 8 | 38.67 | 4.8338 | 38.67 | 37.312 | 0 | 870 |
| map_11 | 7.67 | 44.26 | 5.7705 | 37.26 | 35.686 | 7 | 813 |
| map_12 | 1.47 | 17.3 | 11.7687 | 15.3 | 15.323 | 2 | 129 |
| map_6 | 4.19 | 27.89 | 6.6563 | 25.89 | 26.418 | 2 | 429 |
| map_2 | 7.52 | 35.97 | 4.7832 | 33.97 | 35.255 | 2 | 778.5 |
| map_3 | 8 | 37.47 | 4.6837 | 37.47 | 35.978 | 0 | 818 |
| map_4 | 8 | 36.27 | 4.5338 | 36.27 | 36.682 | 0 | 847 |
| map_5 | 7.56 | 38.17 | 5.0489 | 35.17 | 36.87 | 3 | 829 |
| map_1 | 5.4 | 32.65 | 6.0463 | 31.65 | 30.807 | 1 | 607.5 |

Resumen del paso 3. `Z_sd` es la desviación sobre escenarios (una semilla por escenario), no sobre semillas de una misma instancia:

| instancia | n | Z_media | Z_sd | Z_p5 | Z_p50 | Z_p95 | casos_usual | casos_presupuesto_corto | casos_existentes_de_sobra | paros_tope | paros_sin_mejora |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | 1000 | 564.7676 | 10.6135 | 546.3742 | 565.2736 | 581.1879 | 0 | 1000 | 0 | 0 | 1000 |
| map_27 | 1000 | 733.2819 | 12.0369 | 711.9399 | 734.1824 | 752.5049 | 0 | 1000 | 0 | 0 | 1000 |
| map_28 | 100 | 4075.6706 | 42.6874 | 4008.4172 | 4076.4149 | 4136.543 | 0 | 100 | 0 | 0 | 100 |
| map_30 | 100 | 4147.3519 | 42.8316 | 4075.7243 | 4150.3973 | 4210.7166 | 0 | 100 | 0 | 0 | 100 |
| map_20 | 100 | 794.1546 | 12.2443 | 775.6018 | 794.305 | 809.5169 | 0 | 100 | 0 | 0 | 100 |
| map_21 | 100 | 4938.289 | 46.0655 | 4855.3434 | 4940.7597 | 4998.1591 | 0 | 100 | 0 | 0 | 100 |
| map_22 | 100 | 4823.7817 | 49.7005 | 4733.8633 | 4825.0021 | 4892.1255 | 0 | 100 | 0 | 0 | 100 |
| map_23 | 100 | 3372.4389 | 34.7725 | 3314.5926 | 3369.1445 | 3428.1824 | 0 | 100 | 0 | 0 | 100 |
| map_24 | 100 | 3438.0577 | 35.9348 | 3386.3285 | 3433.9102 | 3498.3377 | 0 | 100 | 0 | 0 | 100 |
| map_18 | 100 | 4374.0983 | 44.7448 | 4295.9756 | 4375.1648 | 4440.0608 | 0 | 100 | 0 | 0 | 100 |
| map_17 | 100 | 3740.9294 | 37.6132 | 3678.2329 | 3740.1497 | 3797.7262 | 0 | 100 | 0 | 0 | 100 |
| map_16 | 100 | 3440.7853 | 36.9589 | 3381.5652 | 3451.7166 | 3498.1236 | 0 | 100 | 0 | 0 | 100 |
| map_19 | 100 | 2995.1673 | 36.1428 | 2943.4876 | 2993.7386 | 3054.1097 | 0 | 100 | 0 | 0 | 100 |
| map_25 | 100 | 3076.0145 | 32.4344 | 3018.4755 | 3077.6675 | 3125.0725 | 0 | 100 | 0 | 0 | 100 |
| map_26 | 100 | 2878.2544 | 32.2789 | 2823.923 | 2878.1978 | 2931.9088 | 0 | 100 | 0 | 0 | 100 |
| map_13 | 100 | 4903.9347 | 53.673 | 4820.9389 | 4908.0046 | 4972.418 | 0 | 100 | 0 | 0 | 100 |
| map_31 | 100 | 4513.0492 | 50.1339 | 4439.9296 | 4519.6964 | 4587.9107 | 0 | 100 | 0 | 0 | 100 |
| map_14 | 100 | 3653.8778 | 37.1442 | 3601.7624 | 3653.7389 | 3708.2231 | 0 | 100 | 0 | 0 | 100 |
| map_15 | 100 | 3292.3565 | 31.8218 | 3239.4881 | 3290.6239 | 3339.2388 | 0 | 100 | 0 | 0 | 100 |
| map_7 | 100 | 3828.4041 | 41.5826 | 3755.519 | 3832.1433 | 3882.2046 | 0 | 100 | 0 | 0 | 100 |
| map_8 | 100 | 4676.8121 | 47.654 | 4591.5784 | 4680.4191 | 4738.6532 | 0 | 100 | 0 | 0 | 100 |
| map_9 | 100 | 4926.3307 | 56.4112 | 4826.971 | 4936.3218 | 4997.0552 | 0 | 100 | 0 | 0 | 100 |
| map_10 | 100 | 4927.902 | 60.1085 | 4820.9611 | 4930.8624 | 5009.8514 | 0 | 100 | 0 | 0 | 100 |
| map_11 | 100 | 4707.9653 | 46.4553 | 4643.2392 | 4711.0621 | 4786.5771 | 0 | 100 | 0 | 0 | 100 |
| map_12 | 100 | 846.4532 | 13.7713 | 822.0253 | 847.5068 | 867.7102 | 0 | 100 | 0 | 0 | 100 |
| map_6 | 100 | 2530.9082 | 27.7687 | 2484.4418 | 2533.7372 | 2575.1187 | 0 | 100 | 0 | 0 | 100 |
| map_2 | 100 | 4631.1091 | 41.734 | 4562.6863 | 4630.5133 | 4695.9541 | 0 | 100 | 0 | 0 | 100 |
| map_3 | 100 | 4931.8211 | 50.2691 | 4847.7731 | 4935.7652 | 5002.1976 | 0 | 100 | 0 | 0 | 100 |
| map_4 | 100 | 4934.9955 | 52.3661 | 4836.6845 | 4946.3433 | 5005.3258 | 0 | 100 | 0 | 0 | 100 |
| map_5 | 100 | 4651.3359 | 44.9798 | 4581.1506 | 4651.0288 | 4714.0405 | 0 | 100 | 0 | 0 | 100 |
| map_1 | 100 | 3288.4573 | 36.036 | 3223.338 | 3292.8129 | 3338.1454 | 0 | 100 | 0 | 0 | 100 |

## Práctica actual

Compra (a nominal 658/ha, b nominal 524/ha, c paso 2 ε = 0, d paso 2 ε = 0.1):

| instancia | area_ha | opcion | descripcion | plantas_compradas | costo_ha | cabe | faltante_medio | sobrante_medio | supervivencia | vivas_1ano |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | 1 | a | nominal 658/ha | 658 | 15473 | False | 0 | 129.467 | 0.9147 | 483.4413 |
| 1ha | 1 | b | nominal 524/ha | 524 | 12324.5 | True | 15.638 | 11.105 | 0.9149 | 469.2484 |
| 1ha | 1 | c | paso 2 (eps=0) | 530 | 12473 | True | 12.264 | 13.731 | 0.9148 | 472.2903 |
| 1ha | 1 | d | paso 2 (eps=0.1) | 530 | 12422 | True | 12.264 | 13.731 | 0.9141 | 471.9109 |
| map_27 | 1.28 | a | nominal 658/ha | 842 | 15471.0938 | False | 0 | 165.332 | 0.9147 | 618.9554 |
| map_27 | 1.28 | b | nominal 524/ha | 671 | 12330.4688 | True | 17.974 | 12.306 | 0.9149 | 602.6338 |
| map_27 | 1.28 | c | paso 2 (eps=0) | 678 | 12470.7031 | True | 13.805 | 15.137 | 0.9148 | 606.4054 |
| map_27 | 1.28 | d | paso 2 (eps=0.1) | 678 | 12415.625 | True | 13.805 | 15.137 | 0.9138 | 605.7462 |

Distribución. `Z_por_planta` divide entre las plantas presentes (T + M), que en las variantes con existentes son menos que D_tot; por eso se añaden Z/D_tot y Z/ha (D_tot y el área vienen de `malla_resumen.csv`):

| instancia | variante | plantas | D_tot | Z | Z_por_planta | Z/D_tot | Z/ha | pares_inmediatos_misma_especie | Z_nivel1 | Z_nivel2 | Z_nivel3 | Z_nivel4 | Z_nivel5 | Z_otro |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | i_serpentina | 658 | 658 | 749.4803 | 1.139 | 1.139 | 749.4803 | 168 | 193.7051 | 216.1505 | 301.8658 | 5.8374 | 31.9215 | 0 |
| 1ha | ii_serpentina_banda | 658 | 658 | 802.7557 | 1.22 | 1.22 | 802.7557 | 226 | 250.7705 | 240.1468 | 275.9186 | 5.541 | 30.3788 | 0 |
| 1ha | iii_serpentina_existentes | 644 | 658 | 773.8551 | 1.2016 | 1.1761 | 773.8551 | 206 | 248.903 | 210.6714 | 278.6455 | 5.0802 | 30.5551 | 0 |
| 1ha | tabu_sin_existentes | 658 | 658 | 569.0427 | 0.8648 | 0.8648 | 569.0427 | 0 | 21.4722 | 180.7613 | 330.4777 | 4.8746 | 31.4569 | 0 |
| 1ha | tabu_con_existentes | 644 | 658 | 561.0573 | 0.8712 | 0.8527 | 561.0573 | 13 | 36.4308 | 185.2749 | 304.167 | 4.4107 | 30.7739 | 0 |
| 1ha | dsatur_sin_existentes | 658 | 658 | 775.6611 | 1.1788 | 1.1788 | 775.6611 | 254 | 328.924 | 128.6861 | 285.1544 | 7.8685 | 25.0281 | 0 |
| 1ha | dsatur_con_existentes | 644 | 658 | 771.3697 | 1.1978 | 1.1723 | 771.3697 | 253 | 320.3116 | 145.5055 | 274.1828 | 7.0062 | 24.3636 | 0 |
| map_27 | i_serpentina | 842 | 842 | 1032.927 | 1.2268 | 1.2268 | 806.9742 | 281 | 347.8633 | 284.7679 | 356.2206 | 4.6789 | 39.3963 | 0 |
| map_27 | ii_serpentina_banda | 842 | 842 | 1072.1232 | 1.2733 | 1.2733 | 837.5962 | 320 | 383.1487 | 312.2077 | 334.7965 | 3.984 | 37.9862 | 0 |
| map_27 | iii_serpentina_existentes | 823 | 842 | 1000.2116 | 1.2153 | 1.1879 | 781.4153 | 279 | 331.9876 | 265.7493 | 358.8671 | 6.3883 | 37.2193 | 0 |
| map_27 | tabu_sin_existentes | 842 | 842 | 738.8838 | 0.8775 | 0.8775 | 577.253 | 0 | 28.6158 | 243.2081 | 419.9312 | 6.2732 | 40.8555 | 0 |
| map_27 | tabu_con_existentes | 823 | 842 | 719.6483 | 0.8744 | 0.8547 | 562.2253 | 28 | 56.3957 | 236.1891 | 381.2185 | 6.4701 | 39.3749 | 0 |
| map_27 | dsatur_sin_existentes | 842 | 842 | 986.6242 | 1.1718 | 1.1718 | 770.8001 | 295 | 400.5847 | 180.3535 | 362.87 | 10.6716 | 32.1444 | 0 |
| map_27 | dsatur_con_existentes | 823 | 842 | 952.7232 | 1.1576 | 1.1315 | 744.315 | 264 | 342.1283 | 208.5082 | 360.0843 | 9.264 | 32.7385 | 0 |

DSATUR por cuotas (§10): en 1ha, Z = 775.7 sin existentes y 771.4 con existentes, contra 569.0 y 561.1 del tabú y 749.5 (i) y 773.9 (iii) de la serpentina (con existentes, DSATUR queda +37.5 % frente al tabú); en map_27, Z = 986.6 sin existentes y 952.7 con existentes, contra 738.9 y 719.6 del tabú y 1032.9 (i) y 1000.2 (iii) de la serpentina (con existentes, DSATUR queda +32.4 % frente al tabú).

Markov. Titular: las filas `compra_d` (compra del paso 2, ε = 0.1) y `compra_nominal` (q = D̄) proyectan la supervivencia sobre las plantas realmente plantadas, E[min(q_j, C⁺_j)] sobre los escenarios. Las filas `D_completo` suponen plantar todo D sin plantas existentes, que ya sobrevivieron al establecimiento, y por eso sobrestiman las vivas y la reposición (entre 24 % y 28 % en la reposición de 1 ha). Las vivas a 1 año de `compra_d` en 1 ha coinciden con la opción d de `practica_compra.csv`.

| instancia | base | composicion | plantas | vivas_1ano | vivas_2anos | reposicion_esperada |
| --- | --- | --- | --- | --- | --- | --- |
| 1ha | compra_d | D* | 516.269 | 471.9109 | 446.9964 | 74.5156 |
| 1ha | compra_nominal | Dnom | 528.533 | 483.4413 | 457.2508 | 76.5748 |
| 1ha | D_completo | D* | 658 | 601.5738 | 569.4941 | 95.1538 |
| 1ha | D_completo | Dnom | 658 | 601.9532 | 569.2745 | 95.2901 |
| map_27 | compra_d | D* | 662.863 | 605.7462 | 573.6475 | 95.98 |
| map_27 | compra_nominal | Dnom | 676.668 | 618.9554 | 585.4385 | 98.0044 |
| map_27 | D_completo | D* | 842 | 769.6294 | 728.5255 | 122.0152 |
| map_27 | D_completo | Dnom | 842 | 770.2886 | 728.5032 | 121.8973 |

## Experimentación (1000 simulaciones)

Valores esperados por hectárea sobre los escenarios (`results/experimentacion.csv`), con IC 95 % = media ± 1.96·sd/√n (sd muestral, n escenarios). X_j son las plantas existentes de la especie j y T = Σ X_j; C_j = max(D*_j − X_j, 0) son las plantas que hay que suministrar para completar D*; q_j es la compra del paso 2 (ε = 0.1, π = 2c), constante; n_j = min(q_j, C_j) son las plantas que realmente se suministran y plantan. Z es la competencia del acomodo del paso 3 (con existentes) y Z_nivel1..5 y Z_otro su reparto por nivel de competencia. Las magnitudes de Monte Carlo usan los 1000 escenarios; Z usa los escenarios del paso 3 (columna n): 1000 en 1 ha y P1 (map_27), mientras que las otras 29 instancias usan 100 escenarios del paso 3 (`paso3_resumen.csv`).

### 1ha

- E[T]/ha (existentes) = 129.47, IC 95 % [129.26, 129.68], n = 1000
- E[Z]/ha (competencia) = 564.77, IC 95 % [564.11, 565.43], n = 1000

Por especie y por ha:

| especie | nombre | E[X_j]/ha | IC X | E[C_j]/ha | IC C | q_j/ha | E[n_j]/ha | IC n |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | Agave lechuguilla | 8.24 | [8.06, 8.42] | 37.76 | [37.58, 37.94] | 38.00 | 36.75 | [36.64, 36.87] |
| 1 | Agave salmiana | 38.22 | [37.89, 38.55] | 139.78 | [139.45, 140.11] | 140.00 | 137.79 | [137.59, 137.99] |
| 2 | Agave scabra | 7.94 | [7.76, 8.11] | 30.06 | [29.89, 30.24] | 30.00 | 28.91 | [28.81, 29.02] |
| 3 | Agave striata | 7.72 | [7.56, 7.88] | 38.28 | [38.12, 38.44] | 38.00 | 37.10 | [37.01, 37.19] |
| 4 | Opuntia cantabrigiensis | 10.19 | [9.99, 10.38] | 42.81 | [42.62, 43.01] | 43.00 | 41.68 | [41.56, 41.80] |
| 5 | Opuntia engelmannii | 8.09 | [7.92, 8.26] | 32.91 | [32.74, 33.08] | 33.00 | 31.86 | [31.76, 31.96] |
| 6 | Opuntia robusta | 15.07 | [14.84, 15.30] | 64.93 | [64.70, 65.16] | 65.00 | 63.47 | [63.33, 63.61] |
| 7 | Opuntia streptacantha | 12.50 | [12.29, 12.71] | 45.50 | [45.29, 45.71] | 46.00 | 44.39 | [44.25, 44.53] |
| 8 | Prosopis laevigata | 16.39 | [16.15, 16.63] | 77.61 | [77.37, 77.85] | 78.00 | 76.25 | [76.10, 76.41] |
| 9 | Yucca filifera | 5.11 | [4.97, 5.25] | 18.89 | [18.75, 19.03] | 19.00 | 18.06 | [17.97, 18.15] |
| total |  | 129.47 |  | 528.53 |  | 530.00 | 516.27 |  |

E[Z por nivel de competencia]/ha:

| magnitud | E/ha | sd/ha | IC 95 % | n |
| --- | --- | --- | --- | --- |
| Z_nivel1 | 34.58 | 4.58 | [34.30, 34.86] | 1000 |
| Z_nivel2 | 187.77 | 6.48 | [187.37, 188.18] | 1000 |
| Z_nivel3 | 307.42 | 5.11 | [307.10, 307.74] | 1000 |
| Z_nivel4 | 4.44 | 0.29 | [4.42, 4.46] | 1000 |
| Z_nivel5 | 30.55 | 0.63 | [30.51, 30.59] | 1000 |
| Z_otro | 0.00 | 0.00 | [0.00, 0.00] | 1000 |

### map_27

- E[T]/ha (existentes) = 129.17, IC 95 % [128.96, 129.37], n = 1000
- E[Z]/ha (competencia) = 572.88, IC 95 % [572.29, 573.46], n = 1000

Por especie y por ha:

| especie | nombre | E[X_j]/ha | IC X | E[C_j]/ha | IC C | q_j/ha | E[n_j]/ha | IC n |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | Agave lechuguilla | 8.17 | [8.01, 8.32] | 37.92 | [37.77, 38.08] | 38.28 | 37.12 | [37.02, 37.23] |
| 1 | Agave salmiana | 38.07 | [37.79, 38.36] | 138.49 | [138.20, 138.77] | 138.28 | 136.57 | [136.40, 136.73] |
| 2 | Agave scabra | 7.88 | [7.73, 8.04] | 30.40 | [30.24, 30.55] | 30.47 | 29.46 | [29.36, 29.56] |
| 3 | Agave striata | 7.77 | [7.62, 7.92] | 38.32 | [38.18, 38.47] | 38.28 | 37.36 | [37.27, 37.45] |
| 4 | Opuntia cantabrigiensis | 10.13 | [9.96, 10.30] | 43.78 | [43.61, 43.95] | 43.75 | 42.69 | [42.58, 42.79] |
| 5 | Opuntia engelmannii | 8.19 | [8.04, 8.34] | 32.44 | [32.29, 32.59] | 32.81 | 31.67 | [31.57, 31.77] |
| 6 | Opuntia robusta | 14.96 | [14.75, 15.16] | 64.73 | [64.53, 64.93] | 64.84 | 63.49 | [63.37, 63.62] |
| 7 | Opuntia streptacantha | 12.64 | [12.44, 12.83] | 45.18 | [44.99, 45.37] | 45.31 | 44.02 | [43.91, 44.14] |
| 8 | Prosopis laevigata | 16.38 | [16.18, 16.58] | 78.15 | [77.95, 78.36] | 78.12 | 76.85 | [76.72, 76.97] |
| 9 | Yucca filifera | 4.98 | [4.86, 5.10] | 19.24 | [19.12, 19.36] | 19.53 | 18.63 | [18.55, 18.71] |
| total |  | 129.17 |  | 528.65 |  | 529.69 | 517.86 |  |

E[Z por nivel de competencia]/ha:

| magnitud | E/ha | sd/ha | IC 95 % | n |
| --- | --- | --- | --- | --- |
| Z_nivel1 | 35.65 | 4.23 | [35.38, 35.91] | 1000 |
| Z_nivel2 | 191.39 | 5.67 | [191.04, 191.74] | 1000 |
| Z_nivel3 | 310.27 | 4.42 | [309.99, 310.54] | 1000 |
| Z_nivel4 | 4.59 | 0.26 | [4.57, 4.60] | 1000 |
| Z_nivel5 | 30.98 | 0.56 | [30.95, 31.02] | 1000 |
| Z_otro | 0.00 | 0.00 | [0.00, 0.00] | 1000 |

## Orden de plantación y transiciones

Ejemplo de orden de plantación en 1 ha, sobre el patrón del paso 3 con ω = 0, en serpentina: filas por `fila` creciente; dentro de la fila, `col` creciente si la fila es par y decreciente si es impar; sólo plantas nuevas (las existentes y las posiciones vacías se saltan). Primeros 21 de 512 pasos (completo en `results/orden_plantacion_1ha.csv`, P1 en `orden_plantacion_map_27.csv`; figuras `orden_plantacion_*.png`):

| paso | pos | fila | col | x | y | especie |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 0 | 0 | 0 | 2.0946 | 1.814 | 1 |
| 2 | 1 | 0 | 1 | 6.2838 | 1.814 | 7 |
| 3 | 3 | 0 | 3 | 14.6621 | 1.814 | 1 |
| 4 | 5 | 0 | 5 | 23.0404 | 1.814 | 7 |
| 5 | 6 | 0 | 6 | 27.2296 | 1.814 | 1 |
| 6 | 7 | 0 | 7 | 31.4188 | 1.814 | 7 |
| 7 | 10 | 0 | 10 | 43.9863 | 1.814 | 1 |
| 8 | 11 | 0 | 11 | 48.1755 | 1.814 | 7 |
| 9 | 12 | 0 | 12 | 52.3646 | 1.814 | 1 |
| 10 | 13 | 0 | 13 | 56.5538 | 1.814 | 7 |
| 11 | 14 | 0 | 14 | 60.743 | 1.814 | 1 |
| 12 | 15 | 0 | 15 | 64.9322 | 1.814 | 6 |
| 13 | 16 | 0 | 16 | 69.1213 | 1.814 | 3 |
| 14 | 17 | 0 | 17 | 73.3105 | 1.814 | 1 |
| 15 | 20 | 0 | 20 | 85.878 | 1.814 | 9 |
| 16 | 21 | 0 | 21 | 90.0672 | 1.814 | 2 |
| 17 | 22 | 0 | 22 | 94.2564 | 1.814 | 7 |
| 18 | 23 | 0 | 23 | 98.4455 | 1.814 | 1 |
| 19 | 46 | 1 | 22 | 96.3509 | 5.4419 | 2 |
| 20 | 45 | 1 | 21 | 92.1618 | 5.4419 | 8 |
| 21 | 43 | 1 | 19 | 83.7834 | 5.4419 | 0 |

### Matrices de transición (1 ha, todos los escenarios del paso 3)

Cada planta es un estado (especie j = 0..9); filas j (de), columnas k (a). Son las matrices principales: acumulan los conteos de todos los escenarios del paso 3. Las de P1 (map_27) están en `results/transicion.csv`.

Por orden de siembra: P(la siguiente planta sembrada es k | la actual es j):

| de \ a | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0.014 | 0.274 | 0.034 | 0.021 | 0.110 | 0.050 | 0.232 | 0.059 | 0.178 | 0.027 |
| 1 | 0.073 | 0.049 | 0.047 | 0.066 | 0.112 | 0.107 | 0.160 | 0.153 | 0.184 | 0.048 |
| 2 | 0.043 | 0.217 | 0.020 | 0.036 | 0.075 | 0.029 | 0.185 | 0.171 | 0.196 | 0.026 |
| 3 | 0.022 | 0.244 | 0.029 | 0.017 | 0.106 | 0.040 | 0.208 | 0.085 | 0.216 | 0.035 |
| 4 | 0.099 | 0.364 | 0.053 | 0.091 | 0.013 | 0.011 | 0.048 | 0.071 | 0.188 | 0.062 |
| 5 | 0.053 | 0.454 | 0.025 | 0.046 | 0.015 | 0.007 | 0.041 | 0.081 | 0.188 | 0.089 |
| 6 | 0.134 | 0.346 | 0.085 | 0.124 | 0.032 | 0.020 | 0.024 | 0.035 | 0.175 | 0.026 |
| 7 | 0.048 | 0.489 | 0.109 | 0.071 | 0.067 | 0.055 | 0.053 | 0.044 | 0.057 | 0.008 |
| 8 | 0.089 | 0.332 | 0.074 | 0.106 | 0.099 | 0.076 | 0.145 | 0.035 | 0.032 | 0.012 |
| 9 | 0.051 | 0.383 | 0.037 | 0.067 | 0.144 | 0.157 | 0.092 | 0.017 | 0.047 | 0.004 |

Por vecindad inmediata: P(un vecino inmediato es k | la planta es j); incluye las existentes:

| de \ a | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0.002 | 0.307 | 0.029 | 0.008 | 0.108 | 0.048 | 0.227 | 0.084 | 0.157 | 0.030 |
| 1 | 0.088 | 0.019 | 0.058 | 0.081 | 0.112 | 0.105 | 0.164 | 0.122 | 0.201 | 0.052 |
| 2 | 0.040 | 0.274 | 0.003 | 0.030 | 0.079 | 0.029 | 0.192 | 0.136 | 0.192 | 0.026 |
| 3 | 0.008 | 0.291 | 0.022 | 0.002 | 0.100 | 0.041 | 0.208 | 0.108 | 0.184 | 0.036 |
| 4 | 0.098 | 0.353 | 0.053 | 0.089 | 0.003 | 0.009 | 0.059 | 0.076 | 0.192 | 0.067 |
| 5 | 0.056 | 0.434 | 0.025 | 0.047 | 0.012 | 0.002 | 0.052 | 0.089 | 0.191 | 0.091 |
| 6 | 0.134 | 0.341 | 0.084 | 0.120 | 0.039 | 0.026 | 0.004 | 0.036 | 0.185 | 0.029 |
| 7 | 0.076 | 0.384 | 0.090 | 0.095 | 0.076 | 0.068 | 0.056 | 0.005 | 0.133 | 0.018 |
| 8 | 0.076 | 0.343 | 0.069 | 0.088 | 0.104 | 0.079 | 0.152 | 0.072 | 0.004 | 0.012 |
| 9 | 0.059 | 0.353 | 0.037 | 0.068 | 0.145 | 0.151 | 0.097 | 0.038 | 0.050 | 0.002 |

Distribución estacionaria de la matriz por vecindad contra la composición plantada (fracción de plantas por especie, existentes incluidas); diferencia absoluta máxima 0.014:

| especie | pi_estacionaria | composicion_plantada | diferencia |
| --- | --- | --- | --- |
| 0 | 0.074 | 0.070 | 0.004 |
| 1 | 0.259 | 0.273 | -0.014 |
| 2 | 0.054 | 0.057 | -0.003 |
| 3 | 0.072 | 0.069 | 0.003 |
| 4 | 0.082 | 0.080 | 0.001 |
| 5 | 0.063 | 0.062 | 0.001 |
| 6 | 0.125 | 0.122 | 0.003 |
| 7 | 0.082 | 0.088 | -0.006 |
| 8 | 0.152 | 0.143 | 0.008 |
| 9 | 0.038 | 0.036 | 0.002 |

### ω = 0 (ejemplo de un solo escenario)

Las mismas matrices sobre el patrón de ω = 0 (el del orden de plantación de arriba), sólo como ejemplo.

Por orden de siembra: P(la siguiente planta sembrada es k | la actual es j):

| de \ a | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0.079 | 0.237 | 0.026 | 0.000 | 0.184 | 0.026 | 0.184 | 0.079 | 0.158 | 0.026 |
| 1 | 0.079 | 0.036 | 0.022 | 0.065 | 0.108 | 0.108 | 0.194 | 0.180 | 0.158 | 0.050 |
| 2 | 0.000 | 0.333 | 0.042 | 0.083 | 0.042 | 0.000 | 0.167 | 0.083 | 0.208 | 0.042 |
| 3 | 0.105 | 0.132 | 0.000 | 0.026 | 0.079 | 0.026 | 0.158 | 0.132 | 0.289 | 0.053 |
| 4 | 0.070 | 0.302 | 0.140 | 0.047 | 0.000 | 0.047 | 0.047 | 0.140 | 0.209 | 0.000 |
| 5 | 0.062 | 0.531 | 0.031 | 0.031 | 0.000 | 0.031 | 0.031 | 0.031 | 0.094 | 0.156 |
| 6 | 0.109 | 0.375 | 0.094 | 0.125 | 0.031 | 0.016 | 0.031 | 0.031 | 0.172 | 0.016 |
| 7 | 0.043 | 0.587 | 0.043 | 0.043 | 0.109 | 0.065 | 0.043 | 0.022 | 0.043 | 0.000 |
| 8 | 0.086 | 0.343 | 0.043 | 0.157 | 0.100 | 0.071 | 0.186 | 0.014 | 0.000 | 0.000 |
| 9 | 0.000 | 0.412 | 0.059 | 0.118 | 0.176 | 0.176 | 0.000 | 0.000 | 0.059 | 0.000 |

Por vecindad inmediata: P(un vecino inmediato es k | la planta es j); incluye las existentes:

| de \ a | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0.000 | 0.335 | 0.037 | 0.004 | 0.094 | 0.053 | 0.229 | 0.078 | 0.135 | 0.037 |
| 1 | 0.088 | 0.011 | 0.064 | 0.074 | 0.107 | 0.108 | 0.174 | 0.111 | 0.208 | 0.054 |
| 2 | 0.043 | 0.284 | 0.019 | 0.033 | 0.090 | 0.019 | 0.204 | 0.123 | 0.152 | 0.033 |
| 3 | 0.004 | 0.278 | 0.028 | 0.008 | 0.081 | 0.036 | 0.202 | 0.137 | 0.202 | 0.024 |
| 4 | 0.084 | 0.364 | 0.069 | 0.073 | 0.000 | 0.004 | 0.055 | 0.091 | 0.200 | 0.062 |
| 5 | 0.057 | 0.441 | 0.017 | 0.039 | 0.004 | 0.009 | 0.048 | 0.092 | 0.197 | 0.096 |
| 6 | 0.122 | 0.353 | 0.094 | 0.109 | 0.033 | 0.024 | 0.004 | 0.031 | 0.196 | 0.035 |
| 7 | 0.065 | 0.350 | 0.088 | 0.116 | 0.085 | 0.071 | 0.048 | 0.020 | 0.136 | 0.020 |
| 8 | 0.061 | 0.356 | 0.059 | 0.092 | 0.101 | 0.083 | 0.165 | 0.073 | 0.000 | 0.011 |
| 9 | 0.065 | 0.360 | 0.050 | 0.043 | 0.122 | 0.158 | 0.115 | 0.043 | 0.043 | 0.000 |

## Sensibilidades

Cada variante cambia un solo parámetro frente a `base`: κ = 0.4 y 0.8 (`kappa_*`), radios × 0.75 y × 1.25 (`radios_*`), m × 0, 0.5 y 2 (`m_*`) y supervivencia a 2 años (`s2`). Cadena completa (paso 1 con γ*, Monte Carlo, paso 2, paso 3 y práctica) en 1 ha y P1 (map_27) con 100 escenarios del paso 3 en todas las variantes, incluida `base` (por eso su Z del paso 3 puede diferir del de la sección del paso 3); en map_21 (8 ha) sólo el paso 1, para comprobar que las conclusiones valen en un polígono grande. `cambio_*_pct` es el cambio porcentual contra `base` en la misma instancia (`results/sensibilidad.csv`; figura `sensibilidad.png`).

| variante | instancia | suma_theta_d | z_opt_media | z_opt_sd | cambio_z_opt_pct | costo_compra | faltante_esperado | z_paso3_media | z_paso3_sd | cambio_z_paso3_pct | z_practica_iii | z_dsatur_con | z_tabu_con |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| base | 1ha | 1344.434 | 571.4363 | 2.9881 | 0 | 12422 | 12.264 | 565.399 | 11.0873 | 0 | 773.8551 | 771.3697 | 561.0573 |
| base | map_27 | 1723.453 | 739.6761 | 0.733 | 0 | 15892 | 13.805 | 734.0388 | 11.1025 | 0 | 1000.2116 | 952.7232 | 719.6483 |
| base | map_21 | 10757.6788 | 4855.1533 | 7.7659 | 0 |  |  |  |  |  |  |  |  |
| kappa_0.4 | 1ha | 1289.6873 | 398.9301 | 1.4631 | -30.1882 | 12414.5 | 12.264 | 404.5467 | 8.2686 | -28.4493 | 598.8711 | 649.7925 | 406.0806 |
| kappa_0.4 | map_27 | 1652.5573 | 515.2921 | 1.3429 | -30.3354 | 15881 | 13.805 | 531.5554 | 9.1859 | -27.5848 | 777.4703 | 798.4897 | 530.3016 |
| kappa_0.4 | map_21 | 10431.675 | 3339.3477 | 7.5019 | -31.2206 |  |  |  |  |  |  |  |  |
| kappa_0.8 | 1ha | 1347.4812 | 758.8425 | 1.0313 | 32.7956 | 12437 | 12.264 | 741.2585 | 14.2338 | 31.1036 | 948.8391 | 904.7901 | 735.3849 |
| kappa_0.8 | map_27 | 1729.9426 | 970.4891 | 2.1166 | 31.2046 | 15897 | 13.805 | 961.2395 | 14.1792 | 30.9521 | 1222.9529 | 1152.1764 | 937.265 |
| kappa_0.8 | map_21 | 10804.4597 | 6387.9973 | 7.1026 | 31.5715 |  |  |  |  |  |  |  |  |
| radios_0.75 | 1ha | 1347.8017 | 510.8662 | 0.8131 | -10.5996 | 12436 | 12.264 | 506.1019 | 10.0086 | -10.4876 | 688.1263 | 686.3109 | 504.8512 |
| radios_0.75 | map_27 | 1727.3146 | 657.6205 | 3.2812 | -11.0935 | 15924 | 13.805 | 655.918 | 10.6437 | -10.6426 | 897.0084 | 851.299 | 648.8015 |
| radios_0.75 | map_21 | 10791.3602 | 4291.7329 | 4.2119 | -11.6046 |  |  |  |  |  |  |  |  |
| radios_1.25 | 1ha | 1304.7588 | 685.362 | 4.0883 | 19.9367 | 12417 | 12.264 | 678.7525 | 13.1222 | 20.0484 | 911.7406 | 857.1883 | 674.359 |
| radios_1.25 | map_27 | 1673.0825 | 876.9075 | 1.4439 | 18.5529 | 15884.5 | 13.805 | 884.9543 | 13.599 | 20.5596 | 1161.2226 | 1080.319 | 857.7506 |
| radios_1.25 | map_21 | 10413.5593 | 5816.8229 | 4.0505 | 19.8072 |  |  |  |  |  |  |  |  |
| m_0 | 1ha | 0 | 572.2638 | 0.8706 | 0.1448 | 12422 | 12.264 | 563.6155 | 11.0684 | -0.3154 | 773.8551 | 755.8919 | 561.1651 |
| m_0 | map_27 | 0 | 736.7895 | 2.504 | -0.3902 | 15892 | 13.805 | 731.8531 | 11.1548 | -0.2978 | 1000.2116 | 959.2301 | 716.9904 |
| m_0 | map_21 | 0 | 4834.8046 | 3.3149 | -0.4191 |  |  |  |  |  |  |  |  |
| m_0.5 | 1ha | 675.4904 | 572.1727 | 1.1553 | 0.1289 | 12422 | 12.264 | 563.3406 | 10.8009 | -0.3641 | 773.8551 | 755.8919 | 560.7347 |
| m_0.5 | map_27 | 866.2834 | 735.9804 | 0.5592 | -0.4996 | 15892 | 13.805 | 732.1672 | 11.0584 | -0.255 | 1000.2116 | 959.2301 | 719.1739 |
| m_0.5 | map_21 | 5414.3242 | 4835.065 | 3.1906 | -0.4138 |  |  |  |  |  |  |  |  |
| m_2 | 1ha | 2559.9312 | 600.5786 | 2.258 | 5.0998 | 12444 | 12.264 | 599.5125 | 11.7193 | 6.0335 | 773.8551 | 822.2396 | 595.3904 |
| m_2 | map_27 | 3265.5875 | 777.7338 | 1.6444 | 5.1452 | 15921 | 13.805 | 781.6569 | 11.9933 | 6.4871 | 1000.2116 | 1028.2391 | 765.9304 |
| m_2 | map_21 | 20488.4144 | 5055.8588 | 9.7837 | 4.1339 |  |  |  |  |  |  |  |  |
| s2 | 1ha | 2079.9107 | 573.2801 | 3.333 | 0.3227 | 12427 | 12.264 | 565.6766 | 11.027 | 0.0491 | 773.8551 | 773.8655 | 561.6218 |
| s2 | map_27 | 2662.3479 | 739.5137 | 0.5622 | -0.022 | 15892 | 13.805 | 734.6306 | 11.2157 | 0.0806 | 1000.2116 | 954.2731 | 718.8017 |
| s2 | map_21 | 16637.5144 | 4857.6546 | 8.6925 | 0.0515 |  |  |  |  |  |  |  |  |

Composición D* por variante:

| variante | instancia | D_0 | D_1 | D_2 | D_3 | D_4 | D_5 | D_6 | D_7 | D_8 | D_9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| base | 1ha | 46 | 178 | 38 | 46 | 53 | 41 | 80 | 58 | 94 | 24 |
| base | map_27 | 59 | 226 | 49 | 59 | 69 | 52 | 102 | 74 | 121 | 31 |
| base | map_21 | 369 | 1412 | 303 | 369 | 431 | 333 | 642 | 461 | 756 | 188 |
| kappa_0.4 | 1ha | 46 | 192 | 38 | 46 | 53 | 41 | 80 | 58 | 79 | 25 |
| kappa_0.4 | map_27 | 59 | 249 | 49 | 59 | 69 | 52 | 102 | 74 | 99 | 30 |
| kappa_0.4 | map_21 | 369 | 1496 | 303 | 369 | 431 | 334 | 642 | 461 | 633 | 226 |
| kappa_0.8 | 1ha | 46 | 177 | 38 | 46 | 53 | 38 | 80 | 58 | 94 | 28 |
| kappa_0.8 | map_27 | 59 | 226 | 49 | 56 | 69 | 51 | 102 | 74 | 121 | 35 |
| kappa_0.8 | map_21 | 369 | 1412 | 303 | 346 | 431 | 317 | 642 | 461 | 756 | 227 |
| radios_0.75 | 1ha | 46 | 178 | 38 | 46 | 53 | 35 | 80 | 60 | 94 | 28 |
| radios_0.75 | map_27 | 59 | 228 | 49 | 59 | 69 | 44 | 102 | 75 | 121 | 36 |
| radios_0.75 | map_21 | 369 | 1412 | 303 | 369 | 431 | 274 | 642 | 480 | 756 | 228 |
| radios_1.25 | 1ha | 46 | 188 | 38 | 46 | 53 | 41 | 80 | 58 | 84 | 24 |
| radios_1.25 | map_27 | 59 | 242 | 49 | 59 | 69 | 52 | 102 | 74 | 106 | 30 |
| radios_1.25 | map_21 | 369 | 1511 | 303 | 369 | 431 | 334 | 642 | 461 | 656 | 188 |
| m_0 | 1ha | 46 | 177 | 38 | 43 | 53 | 41 | 80 | 58 | 94 | 28 |
| m_0 | map_27 | 59 | 226 | 49 | 54 | 69 | 52 | 102 | 74 | 121 | 36 |
| m_0 | map_21 | 369 | 1412 | 303 | 328 | 431 | 334 | 642 | 461 | 756 | 228 |
| m_0.5 | 1ha | 46 | 177 | 38 | 43 | 53 | 41 | 80 | 58 | 94 | 28 |
| m_0.5 | map_27 | 59 | 226 | 49 | 54 | 69 | 52 | 102 | 74 | 121 | 36 |
| m_0.5 | map_21 | 369 | 1412 | 303 | 328 | 431 | 334 | 642 | 461 | 756 | 228 |
| m_2 | 1ha | 46 | 200 | 38 | 46 | 53 | 35 | 80 | 58 | 78 | 24 |
| m_2 | map_27 | 58 | 258 | 49 | 59 | 69 | 44 | 102 | 74 | 99 | 30 |
| m_2 | map_21 | 369 | 1565 | 303 | 369 | 431 | 316 | 642 | 461 | 620 | 188 |
| s2 | 1ha | 46 | 179 | 38 | 46 | 53 | 40 | 80 | 58 | 94 | 24 |
| s2 | map_27 | 59 | 227 | 49 | 59 | 69 | 52 | 102 | 74 | 121 | 30 |
| s2 | map_21 | 369 | 1412 | 303 | 369 | 431 | 333 | 642 | 461 | 756 | 188 |

## Convergencia del tabú

Tope escalado por tamaño de la malla: max_iter_efectivo = max(max_iter, iter_por_posicion·|V|) y max_sin_mejora_efectivo = max(max_sin_mejora, sin_mejora_por_posicion·|V|). Criterio: al menos 90 % de las búsquedas tabú terminan por `sin_mejora` en total y 80 % en cada instancia. Se cumple: 3382 de 3382 búsquedas (100.0 %) y el mínimo por instancia es 100.0 %.

### Motivos de paro

Por instancia, sobre todas las búsquedas del pipeline:

| instancia | busquedas | paros_tope | paros_sin_mejora | frac_sin_mejora |
| --- | --- | --- | --- | --- |
| 1ha | 154 | 0 | 154 | 1 |
| map_27 | 154 | 0 | 154 | 1 |
| map_28 | 106 | 0 | 106 | 1 |
| map_30 | 106 | 0 | 106 | 1 |
| map_20 | 106 | 0 | 106 | 1 |
| map_21 | 106 | 0 | 106 | 1 |
| map_22 | 106 | 0 | 106 | 1 |
| map_23 | 106 | 0 | 106 | 1 |
| map_24 | 106 | 0 | 106 | 1 |
| map_18 | 106 | 0 | 106 | 1 |
| map_17 | 106 | 0 | 106 | 1 |
| map_16 | 106 | 0 | 106 | 1 |
| map_19 | 106 | 0 | 106 | 1 |
| map_25 | 106 | 0 | 106 | 1 |
| map_26 | 106 | 0 | 106 | 1 |
| map_13 | 106 | 0 | 106 | 1 |
| map_31 | 106 | 0 | 106 | 1 |
| map_14 | 106 | 0 | 106 | 1 |
| map_15 | 106 | 0 | 106 | 1 |
| map_7 | 106 | 0 | 106 | 1 |
| map_8 | 106 | 0 | 106 | 1 |
| map_9 | 106 | 0 | 106 | 1 |
| map_10 | 106 | 0 | 106 | 1 |
| map_11 | 106 | 0 | 106 | 1 |
| map_12 | 106 | 0 | 106 | 1 |
| map_6 | 106 | 0 | 106 | 1 |
| map_2 | 106 | 0 | 106 | 1 |
| map_3 | 106 | 0 | 106 | 1 |
| map_4 | 106 | 0 | 106 | 1 |
| map_5 | 106 | 0 | 106 | 1 |
| map_1 | 106 | 0 | 106 | 1 |
| total | 3382 | 0 | 3382 | 1 |

### Corrida de referencia

La corrida de referencia usa topes 10 veces mayores (iter_por_posicion = 3000); `dif_rel` es la diferencia relativa de z_opt_media de la corrida final contra la referencia (criterio: <= 0.5 %): map_27: 0.071 %, map_21: 0.174 %. Con topes 100/4 (iter_por_posicion / sin_mejora_por_posicion) map_21 no pasó esta verificación (0.604 % > 0.5 %); el usuario eligió subir los topes a 300/8, con lo que se cumplen el criterio de paro y la referencia.

| instancia | corrida | candidatos | tenure | iter_por_posicion | sin_mejora_por_posicion | max_iter_efectivo | max_sin_mejora_efectivo | z_opt_media | z_opt_sd | tiempo_medio_s | dif_rel |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| map_27 | final | 128 | 10 | 300 | 8 | 252600 | 6736 | 739.6761 | 0.733 | 0.1949 | 0.0007 |
| map_21 | final | 128 | 10 | 300 | 8 | 1579200 | 42112 | 4855.1533 | 7.7659 | 2.6325 | 0.0017 |
| map_21 | referencia | 128 | 10 | 3000 | 10 | 15792000 | 52640 | 4846.7244 | 5.5576 | 4.4014 | 0 |
| map_27 | referencia | 128 | 10 | 3000 | 10 | 2526000 | 8420 | 739.1494 | 1.2858 | 0.246 | 0 |

### Calibración de K y tenure

Calibración con γ = γ* = 3 en 1 ha y map_21, 5 semillas por combinación. z_rel es z_opt_media entre el mínimo de la rejilla en esa instancia. Regla: menor promedio de z_rel entre las dos instancias; si otra combinación queda a <= 0.001 de ese promedio y es más rápida, se elige la más rápida. Resultado: K = 128, tenure = 10.

| instancia | candidatos | tenure | n_semillas | z_opt_media | z_opt_sd | z_rel | tiempo_medio_s | iter_media | frac_sin_mejora |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1ha | 32 | 10 | 5 | 587.5446 | 2.1169 | 1.0275 | 0.05 | 30829.6 | 1 |
| 1ha | 32 | 15 | 5 | 587.4524 | 1.5687 | 1.0273 | 0.0595 | 27890.2 | 1 |
| 1ha | 32 | 25 | 5 | 588.3126 | 1.6428 | 1.0288 | 0.0477 | 29138 | 1 |
| 1ha | 64 | 10 | 5 | 574.1771 | 2.0695 | 1.0041 | 0.1551 | 44576.2 | 1 |
| 1ha | 64 | 15 | 5 | 574.0766 | 1.3373 | 1.0039 | 0.1511 | 46833 | 1 |
| 1ha | 64 | 25 | 5 | 574.6137 | 1.4655 | 1.0049 | 0.1407 | 36379.6 | 1 |
| 1ha | 128 | 10 | 5 | 571.8365 | 2.2322 | 1 | 0.3958 | 64569.2 | 1 |
| 1ha | 128 | 15 | 5 | 572.5971 | 2.2835 | 1.0013 | 0.3771 | 60196.2 | 1 |
| 1ha | 128 | 25 | 5 | 573.6589 | 2.2155 | 1.0032 | 0.3414 | 53353.6 | 1 |
| map_21 | 32 | 10 | 5 | 4963.1842 | 11.3215 | 1.0239 | 0.8209 | 392150 | 1 |
| map_21 | 32 | 15 | 5 | 4977.1105 | 11.2075 | 1.0268 | 0.6731 | 328759.2 | 1 |
| map_21 | 32 | 25 | 5 | 4967.5366 | 3.5859 | 1.0248 | 0.8898 | 430000.4 | 1 |
| map_21 | 64 | 10 | 5 | 4862.3757 | 5.738 | 1.0031 | 1.8441 | 488951.8 | 1 |
| map_21 | 64 | 15 | 5 | 4854.0864 | 7.9062 | 1.0014 | 2.2897 | 615548.8 | 1 |
| map_21 | 64 | 25 | 5 | 4853.3966 | 7.3955 | 1.0013 | 2.3418 | 595663.6 | 1 |
| map_21 | 128 | 10 | 5 | 4854.887 | 5.9833 | 1.0016 | 6.5883 | 867244.6 | 1 |
| map_21 | 128 | 15 | 5 | 4850.2983 | 7.4385 | 1.0007 | 8.1429 | 1075774.4 | 1 |
| map_21 | 128 | 25 | 5 | 4847.1113 | 7.2244 | 1 | 8.1278 | 1087610.2 | 1 |

Promedio entre instancias:

| candidatos | tenure | z_rel_prom | tiempo_prom_s |
| --- | --- | --- | --- |
| 32 | 10 | 1.0257 | 0.4355 |
| 32 | 15 | 1.0271 | 0.3663 |
| 32 | 25 | 1.0268 | 0.4687 |
| 64 | 10 | 1.0036 | 0.9996 |
| 64 | 15 | 1.0027 | 1.2204 |
| 64 | 25 | 1.0031 | 1.2412 |
| 128 | 10 | 1.0008 | 3.4921 |
| 128 | 15 | 1.001 | 4.26 |
| 128 | 25 | 1.0016 | 4.2346 |

### Parámetros finales del tabú

Leídos de `config/default.json`:

| parámetro | valor |
| --- | --- |
| candidatos | 128 |
| tenure | 10 |
| max_iter | 20000 |
| max_sin_mejora | 3000 |
| iter_por_posicion | 300 |
| sin_mejora_por_posicion | 8 |

## Validación exacta

Se corrió CP-SAT (OR-Tools) sobre 10 instancias pequeñas (7 del paso 3, ec. 24, y 3 del paso 1, ec. 21) con un límite de 600 s y 8 workers por formulación; ninguna se probó óptima en ese tiempo. En las 20 corridas CP-SAT arrancó con la mejor solución del tabú (de 10 semillas) como hint (`AddHint`). Primero se corrió la formulación básica y, donde no cerró, también la ajustada; la tabla tiene ambas filas. Los coeficientes enteros del modelo son round(γ·w·h·10 000) para los pares libre–libre; para los términos libre–fijo, el agregado por (v, j), (θ_j + γ·Σ w·h), redondeado una sola vez; y θ·10 000 en el paso 1. Z*, Z_TS y las brechas usan los coeficientes originales (error de redondeo máximo por coeficiente: 5.0e-05). Antes de comparar, el Z/objetivo de Python de cada solución del tabú se verificó contra el reportado por `tabu-val` de Rust, y el de cada solución de CP-SAT se reevaluó con `reto evaluar-val`; ambos coinciden a 1e-6.

Óptimo probado en 0 de 10 instancias (0 con la formulación básica y 0 con la ajustada).

Sin óptimo probado: p1_a (cota 183.6430, incumbente 202.9292, ajustada), p1_b (cota 244.1465, incumbente 270.1126, ajustada), p1_c (cota 381.7119, incumbente 430.3189, ajustada), p3_a (cota 28.5206, incumbente 34.9464, ajustada), p3_b (cota 48.1291, incumbente 60.2890, ajustada), p3_c (cota 35.0971, incumbente 37.7158, ajustada), p3_d (cota 45.2391, incumbente 49.2267, ajustada), p3_e (cota 73.1512, incumbente 81.6954, ajustada), p3_f (cota 35.0029, incumbente 40.5758, ajustada), p3_g (cota 52.6801, incumbente 62.1755, ajustada). En ellas la brecha se mide contra la cota − tolerancia de redondeo (cota válida para el problema con coeficientes originales), así que es una cota superior de la brecha real; para cada instancia se toma la fila de óptimo probado o, si no la hay, la de mayor cota.

En esas filas la incumbente de CP-SAT queda entre 7.46 % y 25.3 % por encima de la cota, así que la brecha contra la cota está dominada por la debilidad de la cota más que por la calidad de las soluciones. La suma de los peores errores de redondeo por término en una solución (tolerancia del modelo escalado) es a lo más 0.031.

Brecha de la mejor semilla del tabú (contra el óptimo o la cota): máxima 25.4 % (p3_b), media 14.4 %. Brecha de la media de las semillas: máxima 27.2 %, media 14.8 %. Contra la incumbente de CP-SAT, la mejor semilla queda entre 0.00532 % y 0.0977 %.

La incumbente de CP-SAT mejora a la mejor semilla del tabú en 10 de 10 instancias (con los coeficientes originales).

Fracción de semillas que alcanzan el valor de la incumbente (|Z_TS − Z*| ≤ 1e-6·max(1, Z*)) en las instancias sin óptimo probado, contra la incumbente de CP-SAT: p1_a: 0.0, p1_b: 0.0, p1_c: 0.0, p3_a: 0.0, p3_b: 0.0, p3_c: 0.0, p3_d: 0.0, p3_e: 0.0, p3_f: 0.0, p3_g: 0.0. En la tabla esta fracción sólo se muestra con óptimo probado (– si no): sin él no es significativa, porque en las filas básicas la incumbente puede ser la propia solución del hint del tabú.

Paso 1: brecha_z = (γ·Z_TS − γ·Z*)/(γ·Z*), la competencia de la mejor semilla contra la de la incumbente de CP-SAT (no contra el óptimo): p1_a: 0.0103 %, p1_b: 0.0407 %, p1_c: 0.028 %. Especies con L_j < U_j (n_especies_libres): p1_a: 1, p1_b: 1, p1_c: 4. En p1_a, p1_b sólo hay una especie libre y Σ D_j = D_tot está fijo, así que la composición queda determinada y Σθ·D es constante: esas instancias validan sólo el acomodo.

∞ (p3_b, basica): la cota es ≤ 0, así que la brecha relativa contra la cota no está acotada.

Tabla completa (`results/validacion_exacta.csv`; en el paso 1 los z son el objetivo γ·Z + Σθ·D, y z_comp es Z; las brechas de la mejor semilla y de la media usan la referencia válida descrita arriba, por lo que pueden diferir ligeramente del CSV; brecha_z sólo se reporta en el paso 1, porque en el paso 3 el CSV la iguala a la brecha contra la cota y no es comparable; – = no aplica):

| id | modelo | formulacion | n_pos | existentes | n_especies_libres | aristas | variables | z_opt | cota | optimo_probado | estado | tiempo_cpsat_s | z_ts_mejor | z_ts_media | z_ts_sd | tiempo_ts_medio_s | brecha_mejor_% | brecha_media_% | brecha_vs_incumbente_% | frac_semillas_optimo | z_comp_opt | z_comp_ts_mejor | brecha_z_% | error_redondeo_max |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| p1_a | paso1 | basica | 49 | 0 | 1 | 316 | 18814 | 202.9361 | 131.0323 | False | FEASIBLE | 600.1348 | 202.9399 | 203.461 | 0.4093 | 0.0391 | 54.9 | 55.3 | 0.00192 | – | 34.9487 | 34.95 | 0.00371 | 4.8e-05 |
| p1_a | paso1 | ajustada | 49 | 0 | 1 | 316 | 32090 | 202.9292 | 183.643 | False | FEASIBLE | 600.3335 | 202.9399 | 203.461 | 0.4093 | 0.0391 | 10.5 | 10.8 | 0.00532 | – | 34.9464 | 34.95 | 0.0103 | 4.8e-05 |
| p1_b | paso1 | basica | 64 | 0 | 1 | 432 | 25473 | 270.1699 | 166.9491 | False | FEASIBLE | 600.1988 | 270.1699 | 270.5203 | 0.2354 | 0.0591 | 61.8 | 62.1 | 0 | – | 47.0016 | 47.0016 | 0 | 4.8e-05 |
| p1_b | paso1 | ajustada | 64 | 0 | 1 | 432 | 43840 | 270.1126 | 244.1465 | False | FEASIBLE | 600.5754 | 270.1699 | 270.5203 | 0.2354 | 0.0591 | 10.7 | 10.8 | 0.0212 | – | 46.9825 | 47.0016 | 0.0407 | 4.8e-05 |
| p1_c | paso1 | basica | 100 | 0 | 4 | 718 | 41803 | 430.3823 | 196.7282 | False | FEASIBLE | 600.4217 | 430.3823 | 431.738 | 0.8791 | 0.0596 | 119 | 119 | 0 | – | 75.4453 | 75.4453 | 0 | 4.8e-05 |
| p1_c | paso1 | ajustada | 100 | 0 | 4 | 718 | 72800 | 430.3189 | 381.7119 | False | FEASIBLE | 601.4923 | 430.3823 | 431.738 | 0.8791 | 0.0596 | 12.8 | 13.1 | 0.0147 | – | 75.4242 | 75.4453 | 0.028 | 4.8e-05 |
| p3_a | paso3 | basica | 49 | 0 |  | 316 | 18814 | 34.957 | 0.0343 | False | FEASIBLE | 600.1773 | 34.9629 | 35.1711 | 0.1766 | 0.0285 | 1.79e+05 | 1.8e+05 | 0.0168 | – | 34.957 | 34.9629 | – | 4.9e-05 |
| p3_a | paso3 | ajustada | 49 | 0 |  | 316 | 32090 | 34.9464 | 28.5206 | False | FEASIBLE | 600.305 | 34.9629 | 35.1711 | 0.1766 | 0.0285 | 22.7 | 23.4 | 0.0473 | – | 34.9464 | 34.9629 | – | 4.9e-05 |
| p3_b | paso3 | basica | 81 | 0 |  | 566 | 33136 | 60.3187 | 0 | False | FEASIBLE | 600.222 | 60.3352 | 61.1635 | 0.4918 | 0.0357 | ∞ | ∞ | 0.0274 | – | 60.3187 | 60.3352 | – | 4.9e-05 |
| p3_b | paso3 | ajustada | 81 | 0 |  | 566 | 57410 | 60.289 | 48.1291 | False | FEASIBLE | 600.712 | 60.3352 | 61.1635 | 0.4918 | 0.0357 | 25.4 | 27.2 | 0.0766 | – | 60.289 | 60.3352 | – | 4.9e-05 |
| p3_c | paso3 | basica | 49 | 10 |  | 316 | 9066 | 37.7158 | 17.7101 | False | FEASIBLE | 600.105 | 37.7193 | 37.7841 | 0.0894 | 0.0265 | 113 | 113 | 0.00936 | – | 37.7158 | 37.7193 | – | 5.0e-05 |
| p3_c | paso3 | ajustada | 49 | 10 |  | 316 | 16551 | 37.7158 | 35.0971 | False | FEASIBLE | 600.2546 | 37.7193 | 37.7841 | 0.0894 | 0.0265 | 7.5 | 7.69 | 0.00936 | – | 37.7158 | 37.7193 | – | 5.0e-05 |
| p3_d | paso3 | basica | 64 | 13 |  | 432 | 16131 | 49.2329 | 20.4933 | False | FEASIBLE | 600.1281 | 49.2329 | 49.2756 | 0.0305 | 0.0265 | 140 | 141 | 0 | – | 49.2329 | 49.2329 | – | 5.0e-05 |
| p3_d | paso3 | ajustada | 64 | 13 |  | 432 | 27810 | 49.2267 | 45.2391 | False | FEASIBLE | 600.5912 | 49.2329 | 49.2756 | 0.0305 | 0.0265 | 8.86 | 8.96 | 0.0127 | – | 49.2267 | 49.2329 | – | 5.0e-05 |
| p3_e | paso3 | basica | 100 | 20 |  | 718 | 26342 | 81.7564 | 28.4522 | False | FEASIBLE | 600.2151 | 81.7573 | 81.8452 | 0.1908 | 0.042 | 188 | 188 | 0.00116 | – | 81.7564 | 81.7573 | – | 5.0e-05 |
| p3_e | paso3 | ajustada | 100 | 20 |  | 718 | 46000 | 81.6954 | 73.1512 | False | FEASIBLE | 600.5266 | 81.7573 | 81.8452 | 0.1908 | 0.042 | 11.8 | 11.9 | 0.0758 | – | 81.6954 | 81.7573 | – | 5.0e-05 |
| p3_f | paso3 | basica | 64 | 13 |  | 432 | 16131 | 40.5758 | 14.4278 | False | FEASIBLE | 600.1422 | 40.5789 | 40.601 | 0.0111 | 0.0326 | 182 | 182 | 0.00767 | – | 40.5758 | 40.5789 | – | 5.0e-05 |
| p3_f | paso3 | ajustada | 64 | 13 |  | 432 | 27810 | 40.5758 | 35.0029 | False | FEASIBLE | 600.5146 | 40.5789 | 40.601 | 0.0111 | 0.0326 | 16 | 16 | 0.00767 | – | 40.5758 | 40.5789 | – | 5.0e-05 |
| p3_g | paso3 | basica | 100 | 20 |  | 718 | 26342 | 62.2312 | 20.8987 | False | FEASIBLE | 600.2433 | 62.2362 | 62.3842 | 0.2434 | 0.0348 | 198 | 199 | 0.00802 | – | 62.2312 | 62.2362 | – | 5.0e-05 |
| p3_g | paso3 | ajustada | 100 | 20 |  | 718 | 46000 | 62.1755 | 52.6801 | False | FEASIBLE | 600.9307 | 62.2362 | 62.3842 | 0.2434 | 0.0348 | 18.2 | 18.5 | 0.0977 | – | 62.1755 | 62.2362 | – | 5.0e-05 |

## Notas de interpretación

- Curva de compromiso: cada punto usa Σθ·D de la mejor semilla (menor objetivo) y la media de Z_opt sobre todas las semillas, que pueden tener D distinto. La figura omite γ > 100; la tabla de paso 1 sí los incluye.
- Comparación de π: las filas con π = 4 y π = 10 de 1 ha y P1 son idénticas porque el presupuesto es activo en ambos (holgura ≤ $23) y la penalización extra ya no compra nada más. No es un error.

## No incluido / pendiente

Todo lo de la especificación tiene salida en `results/`.


## Figuras

- [Histograma de la densidad ρ con las densidades ajustadas](../figures/hist_rho.png)
- [Requerimiento C_j por especie, 1 ha](../figures/C_por_especie_1ha.png)
- [Requerimiento C_j por especie, P1 (map_27)](../figures/C_por_especie_map_27.png)
- [Curva de compromiso competencia vs. costo, 1 ha](../figures/curva_compromiso_1ha.png)
- [Curva de compromiso competencia vs. costo, P1](../figures/curva_compromiso_map_27.png)
- [Convergencia del tabú del paso 1, 1 ha](../figures/convergencia_1ha.png)
- [Convergencia del tabú del paso 1, P1](../figures/convergencia_map_27.png)
- [Convergencia del tabú del paso 1, map_21 (mayor |V|)](../figures/convergencia_map_21.png)
- [Mapa socia vs. tabú (paso 3, ω = 0), 1 ha](../figures/mapa_1ha.png)
- [Mapa socia vs. tabú (paso 3, ω = 0), P1](../figures/mapa_map_27.png)
- [Práctica actual vs. modelo: compra de plantas](../figures/practica_compra.png)
- [Práctica actual vs. tabú: distribución](../figures/practica_distribucion.png)
- [Orden de plantación en serpentina (paso 3, ω = 0), 1 ha](../figures/orden_plantacion_1ha.png)
- [Orden de plantación en serpentina (paso 3, ω = 0), P1](../figures/orden_plantacion_map_27.png)
- [Sensibilidades: cambio de z_opt (paso 1) y de Z (paso 3) frente a base](../figures/sensibilidad.png)
