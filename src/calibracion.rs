//! Calibración de K (candidatos) y tenure del tabú sobre una rejilla.

use crate::busqueda_tabu::MotivoParo;
use crate::config::ConfigTabu;
use crate::paso1::{resolver_paso1, FilaPaso1, Instancia};

#[derive(Clone, Debug)]
pub struct FilaCalibracion {
    pub instancia: String,
    pub candidatos: usize,
    pub tenure: usize,
    pub n_semillas: usize,
    pub z_opt_media: f64,
    pub z_opt_sd: f64,
    pub z_opt_mejor: f64,
    pub objetivo_media: f64,
    /// Paso 1 + ajuste C.
    pub tiempo_medio_s: f64,
    pub iter_media: f64,
    /// Fracción de las 2 búsquedas × semillas que paró por `sin_mejora`.
    pub frac_sin_mejora: f64,
}

fn resumir(instancia: &str, k: usize, tenure: usize, filas: &[FilaPaso1]) -> FilaCalibracion {
    let n = filas.len() as f64;
    let z: Vec<f64> = filas.iter().map(|f| f.z_opt).collect();
    let media = z.iter().sum::<f64>() / n;
    let sd = if filas.len() > 1 {
        (z.iter().map(|v| (v - media).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
    } else {
        0.0
    };
    let sin_mejora = filas
        .iter()
        .flat_map(|f| [f.motivo_paro_paso1, f.motivo_paro_opt])
        .filter(|m| *m == MotivoParo::SinMejora)
        .count();
    FilaCalibracion {
        instancia: instancia.to_string(),
        candidatos: k,
        tenure,
        n_semillas: filas.len(),
        z_opt_media: media,
        z_opt_sd: sd,
        z_opt_mejor: z.iter().cloned().fold(f64::INFINITY, f64::min),
        objetivo_media: filas.iter().map(|f| f.objetivo_paso1).sum::<f64>() / n,
        tiempo_medio_s: filas
            .iter()
            .map(|f| f.tiempo_paso1_s + f.tiempo_opt_s)
            .sum::<f64>()
            / n,
        iter_media: filas
            .iter()
            .map(|f| (f.iter_paso1 + f.iter_opt) as f64)
            .sum::<f64>()
            / n,
        frac_sin_mejora: sin_mejora as f64 / (2.0 * n),
    }
}

/// Corre la rejilla `candidatos` x `tenures` con cada semilla; una fila por combinación
/// (en orden de K y luego tenure). Las combinaciones se reparten entre hilos.
pub fn calibrar(
    instancia: &Instancia,
    gamma: f64,
    base: &ConfigTabu,
    candidatos: &[usize],
    tenures: &[usize],
    semillas: &[u64],
) -> Vec<FilaCalibracion> {
    let combos: Vec<(usize, usize)> = candidatos
        .iter()
        .flat_map(|&k| tenures.iter().map(move |&t| (k, t)))
        .collect();
    let hilos = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(combos.len().max(1));
    let trozo = combos.len().div_ceil(hilos).max(1);
    let mut salida: Vec<Option<FilaCalibracion>> = vec![None; combos.len()];
    std::thread::scope(|s| {
        for (parte, pedidas) in salida.chunks_mut(trozo).zip(combos.chunks(trozo)) {
            s.spawn(move || {
                for (hueco, &(k, t)) in parte.iter_mut().zip(pedidas) {
                    let cfg = ConfigTabu {
                        candidatos: k,
                        tenure: t,
                        ..base.clone()
                    };
                    let filas: Vec<FilaPaso1> = semillas
                        .iter()
                        .map(|&semilla| {
                            let p = cfg.params_para(semilla, instancia.malla.len());
                            resolver_paso1(instancia, gamma, &p).0
                        })
                        .collect();
                    *hueco = Some(resumir(&instancia.nombre, k, t, &filas));
                }
            });
        }
    });
    salida
        .into_iter()
        .map(|f| f.expect("hilo completó"))
        .collect()
}

/// Elige (K, tenure): menor promedio de z_rel entre instancias; si otra combinación queda a
/// <= 0.001 de ese promedio y su tiempo medio es menor, la más rápida de ellas.
pub fn elegir(filas: &[FilaCalibracion]) -> (usize, usize) {
    let mut instancias: Vec<&str> = filas.iter().map(|f| f.instancia.as_str()).collect();
    instancias.sort();
    instancias.dedup();
    let mins: Vec<f64> = instancias
        .iter()
        .map(|i| {
            filas
                .iter()
                .filter(|f| f.instancia == *i)
                .map(|f| f.z_opt_media)
                .fold(f64::INFINITY, f64::min)
        })
        .collect();
    let mut combos: Vec<(usize, usize)> = filas.iter().map(|f| (f.candidatos, f.tenure)).collect();
    combos.sort();
    combos.dedup();
    // (combinación, z_rel promedio, tiempo medio)
    let puntuadas: Vec<((usize, usize), f64, f64)> = combos
        .into_iter()
        .map(|c| {
            let (mut z, mut t, mut n) = (0.0, 0.0, 0.0);
            for (inst, m) in instancias.iter().zip(&mins) {
                if let Some(f) = filas
                    .iter()
                    .find(|f| f.instancia == *inst && (f.candidatos, f.tenure) == c)
                {
                    z += f.z_opt_media / m;
                    t += f.tiempo_medio_s;
                    n += 1.0;
                }
            }
            (c, z / n, t / n)
        })
        .collect();
    let mejor = puntuadas
        .iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .expect("filas no vacías");
    puntuadas
        .iter()
        .filter(|p| p.1 <= mejor.1 + 0.001 && p.2 < mejor.2)
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .unwrap_or(mejor)
        .0
}
