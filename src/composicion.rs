//! Composición nominal D̄ y banda [L, U] por especie.

/// D_tot = min(|V|, round(densidad·A)).
pub fn d_tot(n_posiciones: usize, area: f64, densidad: u32) -> usize {
    n_posiciones.min((densidad as f64 * area).round() as usize)
}

/// Reparte `d_tot` en proporción a δ con el método del mayor residuo: parte entera de
/// d_tot·δ_j/Σδ y el sobrante a las mayores partes decimales (empates: índice menor).
pub fn composicion_nominal(d_tot: usize, delta: &[u32]) -> Vec<usize> {
    let suma: u64 = delta.iter().map(|&x| x as u64).sum();
    let productos: Vec<u64> = delta.iter().map(|&x| d_tot as u64 * x as u64).collect();
    let mut nominal: Vec<usize> = productos.iter().map(|&p| (p / suma) as usize).collect();
    let sobrante = d_tot - nominal.iter().sum::<usize>();

    let mut orden: Vec<usize> = (0..delta.len()).collect();
    orden.sort_by(|&a, &b| {
        (productos[b] % suma)
            .cmp(&(productos[a] % suma))
            .then(a.cmp(&b))
    });
    for &j in orden.iter().take(sobrante) {
        nominal[j] += 1;
    }
    nominal
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Banda {
    pub l: Vec<usize>,
    pub u: Vec<usize>,
}

/// L_j = ceil((1 − ε)·D̄_j), U_j = floor((1 + ε)·D̄_j).
pub fn banda(nominal: &[usize], epsilon: f64) -> Banda {
    Banda {
        l: nominal
            .iter()
            .map(|&x| ((1.0 - epsilon) * x as f64).ceil() as usize)
            .collect(),
        u: nominal
            .iter()
            .map(|&x| ((1.0 + epsilon) * x as f64).floor() as usize)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DELTA: [u32; 10] = [42, 196, 42, 42, 49, 38, 73, 64, 86, 26];

    #[test]
    fn d_tot_es_el_minimo_entre_posiciones_y_658a() {
        assert_eq!(d_tot(837, 1.28, 658), 837);
        assert_eq!(d_tot(658, 1.0, 658), 658);
        assert_eq!(d_tot(5265, 8.0, 658), 5264);
    }

    #[test]
    fn mayor_residuo_con_658_reproduce_delta() {
        let nominal = composicion_nominal(658, &DELTA);
        assert_eq!(nominal, vec![42, 196, 42, 42, 49, 38, 73, 64, 86, 26]);
    }

    #[test]
    fn mayor_residuo_para_837_reparte_el_sobrante_con_desempate_por_indice() {
        // Pisos 53,249,53,53,62,48,92,81,109,33 (suma 833); el sobrante de 4 va a las
        // fracciones .859 (idx 6) y .4255 (idx 0, 2, 3) — calculado a mano.
        assert_eq!(
            composicion_nominal(837, &DELTA),
            vec![54, 249, 54, 54, 62, 48, 93, 81, 109, 33]
        );
    }

    #[test]
    fn mayor_residuo_con_d_tot_cero() {
        assert_eq!(composicion_nominal(0, &DELTA), vec![0; 10]);
    }

    #[test]
    fn banda_usa_ceil_abajo_y_floor_arriba() {
        let b = banda(&[42, 10, 196, 0], 0.10);
        assert_eq!(b.l, vec![38, 9, 177, 0]);
        assert_eq!(b.u, vec![46, 11, 215, 0]);
    }

    #[test]
    fn banda_con_epsilon_cero_es_la_nominal() {
        let b = banda(&[54, 249, 33], 0.0);
        assert_eq!(b.l, vec![54, 249, 33]);
        assert_eq!(b.u, vec![54, 249, 33]);
    }
}
