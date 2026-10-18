use std::path::Path;
use reforestation::datos::{Datos, N_ESPECIES};
use reforestation::paso2::{resolver_ple, EntradaPaso2, voceador, necesidad};
use reforestation::paso1::Instancia;
use reforestation::config::Config;
use reforestation::escenarios::{generar, ParamsMc};

fn main() {
    let datos = Datos::cargar(Path::new("data")).unwrap();
    let config = Config::cargar(Path::new("config/default.json")).unwrap();
    let mc = ParamsMc::leer(Path::new("results/mc_params.json")).unwrap();
    let mut lector = csv::Reader::from_path("results/paso1_composicion.csv").unwrap();
    let enc = lector.headers().unwrap().clone();
    let ci = enc.iter().position(|h| h == "instancia").unwrap();
    let carea = enc.iter().position(|h| h == "area_ha").unwrap();
    let cols: Vec<usize> = (0..N_ESPECIES).map(|j| enc.iter().position(|h| h == format!("D_{j}")).unwrap()).collect();
    for fila in lector.records() {
        let fila = fila.unwrap();
        if &fila[ci] != "map_1" { continue; }
        let area: f64 = fila[carea].parse().unwrap();
        let mut d = [0u32; N_ESPECIES];
        for j in 0..N_ESPECIES { d[j] = fila[cols[j]].parse::<f64>().unwrap().round() as u32; }
        let inst = Instancia::nueva("map_1", area, &datos, &config, config.m_escala, config.usar_s2);
        let x: Vec<[u32; N_ESPECIES]> = (0..config.n_escenarios).map(|w| generar(inst.malla.len(), area, inst.map_id, w, config.semilla, &mc).x).collect();
        let c = std::array::from_fn(|j| datos.especies[j].c);
        println!("area={area} presupuesto={:.0}", config.presupuesto_ha * area);
        println!("c ={:?}", c);
        println!("D*={:?}", d);
        let mut cmax=[0u32;N_ESPECIES];
        for j in 0..N_ESPECIES { for f in &x { cmax[j]=cmax[j].max(necesidad(&d,f)[j]); } }
        println!("cmax={:?}", cmax);
        let pi = std::array::from_fn(|j| 2.0*c[j]);
        let e = EntradaPaso2{d:&d,x:&x,c:&c,pi:&pi,presupuesto:None,eta:0.0};
        let libre = resolver_ple(&e).unwrap();
        println!("libre q={:?} costo={:.2}", libre.q, libre.costo);
        let qv = voceador(&c,&pi,0.0,&d,&x);
        println!("voceador q={:?} costo={:.2}", qv, (0..N_ESPECIES).map(|j| c[j]*qv[j] as f64).sum::<f64>());
        break;
    }
}
