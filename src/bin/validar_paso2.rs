//! Verificación del PLE de Rust contra `results/paso2_compra.csv` (escrito por Python/PuLP).
//! Se corre con `cargo run --bin validar_paso2`. No forma parte del modelo.

use std::path::Path;

use reforestation::config::Config;
use reforestation::datos::{Datos, N_ESPECIES};
use reforestation::escenarios::{generar, ParamsMc};
use reforestation::paso1::Instancia;
use reforestation::paso2::{resolver_ple, EntradaPaso2};

fn columnas(enc: &csv::StringRecord) -> Vec<usize> {
    (0..N_ESPECIES)
        .map(|j| enc.iter().position(|h| h == format!("D_{j}")).unwrap())
        .collect()
}

fn main() {
    let datos = Datos::cargar(Path::new("data")).expect("data/");
    let config = Config::cargar(Path::new("config/default.json")).expect("config");
    let mc = ParamsMc::leer(Path::new("results/mc_params.json")).unwrap();

    let mut lector = csv::Reader::from_path("results/paso1_composicion.csv").unwrap();
    let enc = lector.headers().unwrap().clone();
    let ci = enc.iter().position(|h| h == "instancia").unwrap();
    let carea = enc.iter().position(|h| h == "area_ha").unwrap();
    let cols = columnas(&enc);

    let mut lector2 = csv::Reader::from_path("results/paso2_compra.csv").unwrap();
    let enc2 = lector2.headers().unwrap().clone();
    let c2 = |n: &str| enc2.iter().position(|h| h == n).unwrap();
    let (ci2, ce2, cp2) = (c2("instancia"), c2("epsilon"), c2("pi_factor"));
    let cq: Vec<usize> = (0..N_ESPECIES).map(|j| c2(&format!("q_{j}"))).collect();
    let cqv: Vec<usize> = (0..N_ESPECIES)
        .map(|j| c2(&format!("q_voceador_{j}")))
        .collect();
    let ccosto = c2("costo");
    let cobj = c2("objetivo");

    let mut filas_pub = Vec::new();
    for r in lector2.records() {
        let r = r.unwrap();
        filas_pub.push(r);
    }

    let mut ok = 0;
    let mut mal = 0;
    let mut difs = 0.0f64;

    for fila in lector.records() {
        let fila = fila.unwrap();
        let nombre = fila[ci].to_string();
        let area: f64 = fila[carea].parse().unwrap();
        let mut d = [0u32; N_ESPECIES];
        for j in 0..N_ESPECIES {
            d[j] = fila[cols[j]].parse::<f64>().unwrap().round() as u32;
        }
        let inst = Instancia::nueva(
            &nombre,
            area,
            &datos,
            &config,
            config.m_escala,
            config.usar_s2,
        );
        let escenarios: Vec<_> = (0..config.n_escenarios)
            .map(|w| generar(inst.malla.len(), area, inst.map_id, w, config.semilla, &mc))
            .collect();
        let x: Vec<[u32; N_ESPECIES]> = escenarios.iter().map(|e| e.x).collect();
        let c = std::array::from_fn(|j| datos.especies[j].c);

        for pub_f in &filas_pub {
            if pub_f[ci2] != nombre {
                continue;
            }
            let eps: f64 = pub_f[ce2].parse().unwrap();
            let pf: f64 = pub_f[cp2].parse().unwrap();
            if (eps - 0.10).abs() > 1e-9 {
                continue;
            }
            let pi = std::array::from_fn(|j| pf * c[j]);
            let entrada = EntradaPaso2 {
                d: &d,
                x: &x,
                c: &c,
                pi: &pi,
                presupuesto: Some(config.presupuesto_ha * area),
                eta: 0.0,
            };
            let res = resolver_ple(&entrada).unwrap();
            let q_pub: Vec<u32> = cq
                .iter()
                .map(|&k| pub_f[k].parse::<f64>().unwrap().round() as u32)
                .collect();
            let qv_pub: Vec<u32> = cqv
                .iter()
                .map(|&k| pub_f[k].parse::<f64>().unwrap().round() as u32)
                .collect();
            let costo_pub: f64 = pub_f[ccosto].parse().unwrap();
            let obj_pub: f64 = pub_f[cobj].parse().unwrap();
            let igual: bool = res.q.to_vec() == q_pub;
            let igual_v: bool = res.q_voceador.to_vec() == qv_pub;
            difs += (res.costo - costo_pub).abs();
            println!("{nombre:>10} e={eps} pi={pf:>4}: q {} (pub {}, igual={igual}) | voceador igual={igual_v} | costo {:.2} vs pub {:.2} | obj {:.2} vs {:.2}", res.q.iter().sum::<u32>(), q_pub.iter().sum::<u32>(), res.costo, costo_pub, res.objetivo, obj_pub);
            if igual && igual_v {
                ok += 1
            } else {
                mal += 1
            }
        }
    }
    println!(
        "\nok={ok} mal={mal} dif media costo={:.4}",
        difs / (ok + mal).max(1) as f64
    );
}
