//! Binario del reto CONAFOR (Entregable 2).
//!
//! Uso: `reto <subcomando> [--datos DIR] [--resultados DIR]`
//!   datos   escribe `DIR/*.csv` con los datos del reto
//!   malla   |V|, D_tot y pares por anillo de 1 ha y de cada polígono; exporta W

use std::path::Path;
use std::process::ExitCode;

use reforestation::competencia::matriz_w;
use reforestation::config::Config;
use reforestation::datos::Datos;
use reforestation::reportes::{
    escribir_anillos, escribir_resumen_malla, escribir_w, resumen_malla,
};

/// Configuración compartida con Python; si no existe se usan los valores por defecto.
const CONFIG_POR_DEFECTO: &str = "config/default.json";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match ejecutar(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn opcion(args: &[String], nombre: &str, por_defecto: &str) -> String {
    args.windows(2)
        .find(|par| par[0] == nombre)
        .map(|par| par[1].clone())
        .unwrap_or_else(|| por_defecto.to_string())
}

fn ejecutar(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let datos_dir = opcion(args, "--datos", "data");
    let resultados_dir = opcion(args, "--resultados", "results");
    match args.get(1).map(String::as_str) {
        Some("datos") => {
            Datos::base().escribir(Path::new(&datos_dir))?;
            println!("datos escritos en {datos_dir}");
            Ok(())
        }
        Some("malla") => {
            let mut config = match args.windows(2).find(|par| par[0] == "--config") {
                Some(par) => Config::cargar(Path::new(&par[1]))?,
                None if Path::new(CONFIG_POR_DEFECTO).exists() => {
                    Config::cargar(Path::new(CONFIG_POR_DEFECTO))?
                }
                None => Config::default(),
            };
            for par in args.windows(2).filter(|par| par[0] == "--set") {
                config.aplicar(&par[1])?;
            }
            malla(Path::new(&datos_dir), Path::new(&resultados_dir), &config)
        }
        otro => Err(format!("subcomando desconocido: {}", otro.unwrap_or("")).into()),
    }
}

fn malla(
    datos_dir: &Path,
    resultados_dir: &Path,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let datos = Datos::cargar(datos_dir)?;
    std::fs::create_dir_all(resultados_dir)?;

    let filas = resumen_malla(&datos, config);
    escribir_resumen_malla(&resultados_dir.join("malla_resumen.csv"), &filas)?;
    escribir_anillos(&resultados_dir.join("malla_anillos.csv"), &filas, config.d)?;
    let w = matriz_w(&datos.especies, config.kappa);
    escribir_w(&resultados_dir.join("W.csv"), &datos.especies, &w)?;

    println!("d = {:.4} m", config.d);
    for fila in filas.iter().take(2) {
        println!(
            "{:<8} A = {:.2} ha  |V| = {}  round(658A) = {}  dif = {}  D_tot = {}",
            fila.instancia, fila.area, fila.n_pos, fila.objetivo, fila.diferencia, fila.d_tot
        );
        for (k, (distancia, pares)) in fila.anillos_m.iter().zip(&fila.pares).enumerate() {
            println!(
                "    anillo {}: {:.3} m ({:.3}·d)  {} pares",
                k + 1,
                distancia,
                distancia / config.d,
                pares
            );
        }
    }
    Ok(())
}
