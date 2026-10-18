//! Paso 1: diseño de la composición D* y del patrón base φ* sobre la malla sin existentes.

use crate::busqueda_tabu::{buscar, ParamsTabu, Problema, Restriccion};
use crate::competencia::{matriz_w, radio_max, radios, TablaCompetencia};
use crate::composicion::{banda, composicion_nominal, Banda};
use crate::config::Config;
use crate::datos::{Datos, Especie, N_ESPECIES};
use crate::malla::Malla;
use crate::objetivo::Grafo;
use crate::tabu::DEFAULT_ROW_SEQUENCE;

/// Lo que el paso 1 necesita de un polígono (o de 1 ha).
pub struct Instancia {
    pub nombre: String,
    /// map_id del polígono (0 para 1 ha).
    pub map_id: u32,
    pub area: f64,
    pub malla: Malla,
    pub d_tot: usize,
    /// Composición nominal D̄ de D_tot.
    pub nominal: Vec<usize>,
    /// [L_j, U_j] alrededor de la composición nominal D̄ de D_tot.
    pub banda: Banda,
    pub grafo: Grafo,
    pub tabla: TablaCompetencia,
    /// θ_j = m_escala·c_j·(1 − s_j).
    pub theta: [f64; N_ESPECIES],
    /// c_j de cada especie (sólo se reporta; no entra al objetivo del paso 1).
    pub costos: [f64; N_ESPECIES],
}

impl Instancia {
    pub fn nueva(
        nombre: &str,
        area: f64,
        datos: &Datos,
        config: &Config,
        m_escala: f64,
        usar_s2: bool,
    ) -> Instancia {
        let d_tot = (datos.densidad_total() as f64 * area).round() as usize;
        let malla = Malla::para_objetivo(d_tot, config.d);
        let r = radios(&datos.especies, config.d, config.factor_radios);
        let vecindad = malla.vecindad(radio_max(&r, config.d), config.tolerancia);
        let w = matriz_w(&datos.especies, config.kappa);
        let nominal = composicion_nominal(
            d_tot,
            &datos.especies.iter().map(|e| e.delta).collect::<Vec<_>>(),
        );
        let map_id = nombre
            .strip_prefix("map_")
            .and_then(|m| m.parse().ok())
            .unwrap_or(0);
        Instancia {
            nombre: nombre.to_string(),
            map_id,
            area,
            grafo: Grafo::desde(&vecindad),
            tabla: TablaCompetencia::nueva(&w, &r, &vecindad.anillos, config.d, config.tolerancia),
            malla,
            d_tot,
            banda: banda(&nominal, config.epsilon),
            nominal,
            theta: theta(&datos.especies, m_escala, usar_s2),
            costos: std::array::from_fn(|j| datos.especies[j].c),
        }
    }
}

/// Resultado de una corrida (γ, semilla) del paso 1 más el ajuste C.
#[derive(Clone, Debug, PartialEq)]
pub struct FilaPaso1 {
    pub instancia: String,
    pub map_id: u32,
    pub gamma: f64,
    pub semilla: u64,
    pub d: [usize; N_ESPECIES],
    pub d_tot: usize,
    pub suma_theta_d: f64,
    pub suma_c_d: f64,
    pub z_paso1: f64,
    /// Valor del objetivo del paso 1 (γ·Z + Σθ·D) en la mejor solución del paso 1.
    pub objetivo_paso1: f64,
    pub z_opt: f64,
    pub z_opt_por_planta: f64,
    pub tiempo_paso1_s: f64,
    pub iter_paso1: usize,
    pub tiempo_opt_s: f64,
    pub iter_opt: usize,
    /// Patrón φ* final (después del ajuste C).
    pub phi: Vec<u8>,
    /// Objetivo del arreglo actual por iteración del paso 1.
    pub traza: Vec<f64>,
    /// Mejor objetivo del paso 1 por iteración.
    pub curva: Vec<f64>,
}

impl FilaPaso1 {
    /// Fila en ceros (para armar filas de prueba con `..FilaPaso1::vacia()`).
    pub fn vacia() -> FilaPaso1 {
        FilaPaso1 {
            instancia: String::new(),
            map_id: 0,
            gamma: 0.0,
            semilla: 0,
            d: [0; N_ESPECIES],
            d_tot: 0,
            suma_theta_d: 0.0,
            suma_c_d: 0.0,
            z_paso1: 0.0,
            objetivo_paso1: 0.0,
            z_opt: 0.0,
            z_opt_por_planta: 0.0,
            tiempo_paso1_s: 0.0,
            iter_paso1: 0,
            tiempo_opt_s: 0.0,
            iter_opt: 0,
            phi: Vec::new(),
            traza: Vec::new(),
            curva: Vec::new(),
        }
    }
}

/// Paso 1 (min γ·Z + Σθ·D en la banda) seguido del ajuste C (sólo intercambios, D* fija).
/// La semilla de la búsqueda es `params.semilla`.
pub fn resolver_paso1(
    instancia: &Instancia,
    gamma: f64,
    params: &ParamsTabu,
) -> (FilaPaso1, Vec<u8>) {
    let mut phi = patron_serpentina(&instancia.malla, DEFAULT_ROW_SEQUENCE, instancia.d_tot);
    reparar_a_banda(&mut phi, &instancia.banda);
    let libres = vec![false; phi.len()];
    let composicion = Problema {
        grafo: &instancia.grafo,
        tabla: &instancia.tabla,
        fija: libres.clone(),
        gamma,
        theta: instancia.theta,
        restriccion: Restriccion::Banda {
            l: instancia.banda.l.clone(),
            u: instancia.banda.u.clone(),
        },
    };
    let paso1 = buscar(&composicion, phi, params);
    // Ajuste C: sólo intercambios (composición D* fija) para minimizar Z.
    let solo_z = Problema {
        grafo: &instancia.grafo,
        tabla: &instancia.tabla,
        fija: libres,
        gamma: 1.0,
        theta: [0.0; N_ESPECIES],
        restriccion: Restriccion::ComposicionFija,
    };
    let ajuste = buscar(&solo_z, paso1.phi.clone(), params);
    let mut d = [0usize; N_ESPECIES];
    for &c in ajuste.phi.iter().filter(|&&c| c > 0) {
        d[c as usize - 1] += 1;
    }
    let suma = |v: &[f64; N_ESPECIES]| (0..N_ESPECIES).map(|j| v[j] * d[j] as f64).sum::<f64>();
    (
        FilaPaso1 {
            instancia: instancia.nombre.clone(),
            map_id: instancia.map_id,
            gamma,
            semilla: params.semilla,
            d,
            d_tot: instancia.d_tot,
            suma_theta_d: suma(&instancia.theta),
            suma_c_d: suma(&instancia.costos),
            z_paso1: paso1.z,
            objetivo_paso1: paso1.objetivo,
            z_opt: ajuste.z,
            z_opt_por_planta: ajuste.z / instancia.d_tot as f64,
            tiempo_paso1_s: paso1.tiempo_s,
            iter_paso1: paso1.iteraciones,
            tiempo_opt_s: ajuste.tiempo_s,
            iter_opt: ajuste.iteraciones,
            phi: ajuste.phi.clone(),
            traza: paso1.traza,
            curva: paso1.curva,
        },
        ajuste.phi,
    )
}

/// Barrido de γ con `semillas`; ver `barrido_con`.
pub fn barrido_gamma(
    instancia: &Instancia,
    gammas: &[f64],
    semillas: &[u64],
    params: &ParamsTabu,
) -> Vec<FilaPaso1> {
    barrido_con(gammas, semillas, |gamma, semilla| {
        let params = ParamsTabu { semilla, ..*params };
        resolver_paso1(instancia, gamma, &params).0
    })
}

/// Mejor semilla de un γ: la de menor objetivo del paso 1.
pub fn mejor_fila<'a>(filas: impl Iterator<Item = &'a FilaPaso1>) -> Option<&'a FilaPaso1> {
    filas.min_by(|a, b| a.objetivo_paso1.total_cmp(&b.objetivo_paso1))
}

/// Corre `resolver(γ, semilla)` para cada γ (en orden creciente) y semilla. Si la D* de la mejor
/// semilla cambia entre los dos γ mayores, agrega γ_max·3, hasta 3 veces.
pub fn barrido_con(
    gammas: &[f64],
    semillas: &[u64],
    mut resolver: impl FnMut(f64, u64) -> FilaPaso1,
) -> Vec<FilaPaso1> {
    let mut gammas = gammas.to_vec();
    gammas.sort_by(f64::total_cmp);
    let mut filas = Vec::new();
    let mut correr = |gamma: f64, filas: &mut Vec<FilaPaso1>| {
        for &semilla in semillas {
            filas.push(resolver(gamma, semilla));
        }
    };
    for &gamma in &gammas {
        correr(gamma, &mut filas);
    }
    let mejor_d = |filas: &[FilaPaso1], gamma: f64| {
        mejor_fila(filas.iter().filter(|f| f.gamma == gamma)).map(|f| f.d)
    };
    for _ in 0..3 {
        let n = gammas.len();
        if n < 2 || gammas[n - 1] <= 0.0 {
            break;
        }
        if mejor_d(&filas, gammas[n - 1]) == mejor_d(&filas, gammas[n - 2]) {
            break;
        }
        let nuevo = gammas[n - 1] * 3.0;
        correr(nuevo, &mut filas);
        gammas.push(nuevo);
    }
    filas
}

/// Reasigna plantas hasta que L_j ≤ D_j ≤ U_j; las vacías no se tocan.
pub fn reparar_a_banda(phi: &mut [u8], banda: &Banda) {
    let mut d = [0usize; N_ESPECIES + 1];
    for &c in phi.iter() {
        d[c as usize] += 1;
    }
    // Cada especie bajo L recibe una planta de alguna especie que esté sobre su L.
    while let Some(j) = (0..N_ESPECIES).find(|&j| d[j + 1] < banda.l[j]) {
        if let Some(u) = (0..phi.len())
            .find(|&u| phi[u] > 0 && d[phi[u] as usize] > banda.l[phi[u] as usize - 1])
        {
            d[phi[u] as usize] -= 1;
            phi[u] = j as u8 + 1;
            d[j + 1] += 1;
        }
    }
    // La prueba también exige D_j ≤ U_j (especie 4 queda en 61 > U = 53): cada especie sobre U
    // cede una planta a alguna especie que esté bajo su U.
    while let Some(k) = (0..N_ESPECIES).find(|&k| d[k + 1] > banda.u[k]) {
        if let Some(j) = (0..N_ESPECIES).find(|&j| d[j + 1] < banda.u[j]) {
            let u = (0..phi.len())
                .find(|&u| phi[u] as usize == k + 1)
                .expect("la especie sobre U tiene plantas");
            d[k + 1] -= 1;
            phi[u] = j as u8 + 1;
            d[j + 1] += 1;
        }
    }
}

/// Secuencia de la socia (índices de especie 0..9) repetida en serpentina: filas pares de
/// izquierda a derecha y filas impares de derecha a izquierda. Se plantan las primeras `d_tot`
/// posiciones del recorrido; las del excedente quedan vacías (φ = 0).
pub fn patron_serpentina(malla: &Malla, secuencia: &[usize], d_tot: usize) -> Vec<u8> {
    let mut phi = vec![0u8; malla.len()];
    let recorrido = (0..malla.filas()).flat_map(|fila| {
        let posiciones: Vec<usize> = malla.fila(fila).collect();
        if fila % 2 == 0 {
            posiciones
        } else {
            posiciones.into_iter().rev().collect()
        }
    });
    for (k, u) in recorrido.take(d_tot).enumerate() {
        phi[u] = secuencia[k % secuencia.len()] as u8 + 1;
    }
    phi
}

/// θ_j = m_j·(1 − s_j) con m_j = m_escala·c_j y s_j = s1_j (o s2_j).
pub fn theta(especies: &[Especie], m_escala: f64, usar_s2: bool) -> [f64; N_ESPECIES] {
    std::array::from_fn(|j| {
        let s = if usar_s2 {
            especies[j].s2
        } else {
            especies[j].s1
        };
        m_escala * especies[j].c * (1.0 - s)
    })
}
