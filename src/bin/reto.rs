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
    escribir_alcance, escribir_anillos, escribir_resumen_malla, escribir_w, resumen_malla,
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

/// Valor de `nombre VALOR`; error si la opción aparece sin valor.
fn opcion(args: &[String], nombre: &str, por_defecto: &str) -> Result<String, String> {
    match args.iter().position(|a| a == nombre) {
        None => Ok(por_defecto.to_string()),
        Some(i) => args
            .get(i + 1)
            .filter(|valor| !valor.starts_with("--"))
            .cloned()
            .ok_or_else(|| format!("{nombre} requiere un valor")),
    }
}

const OPCIONES: [&str; 4] = ["--datos", "--resultados", "--config", "--set"];

/// Tras el subcomando sólo se aceptan pares `--opción valor` con opciones conocidas.
fn validar_opciones(args: &[String]) -> Result<(), String> {
    let mut resto = args.iter().skip(2);
    while let Some(nombre) = resto.next() {
        if !OPCIONES.contains(&nombre.as_str()) {
            return Err(format!("opción desconocida: {nombre}"));
        }
        match resto.next() {
            Some(valor) if !valor.starts_with("--") => {}
            _ => return Err(format!("{nombre} requiere un valor")),
        }
    }
    Ok(())
}

fn ejecutar(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    validar_opciones(args)?;
    let datos_dir = opcion(args, "--datos", "data")?;
    let resultados_dir = opcion(args, "--resultados", "results")?;
    match args.get(1).map(String::as_str) {
        Some("datos") => {
            Datos::base().escribir(Path::new(&datos_dir))?;
            println!("datos escritos en {datos_dir}");
            Ok(())
        }
        Some("malla") => {
            // Se lee sin validar, se aplican todos los --set y se valida una sola vez.
            let mut config = match args.windows(2).find(|par| par[0] == "--config") {
                Some(par) => Config::leer(Path::new(&par[1]))?,
                None if Path::new(CONFIG_POR_DEFECTO).exists() => {
                    Config::leer(Path::new(CONFIG_POR_DEFECTO))?
                }
                None => Config::default(),
            };
            let asignaciones: Vec<&str> = args
                .windows(2)
                .filter(|par| par[0] == "--set")
                .map(|par| par[1].as_str())
                .collect();
            config.aplicar_todas(&asignaciones)?;
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
    escribir_alcance(
        &resultados_dir.join("alcance_h.csv"),
        &datos.especies,
        config,
        &filas[0].anillos_m,
    )?;
    let w = matriz_w(&datos.especies, config.kappa);
    escribir_w(&resultados_dir.join("W.csv"), &datos.especies, &w)?;

    println!("d = {:.4} m", config.d);
    for fila in filas.iter().take(2) {
        println!(
            "{:<8} A = {:.2} ha  {} x {} ({:.1} x {:.1} m)  |V| = {}  round(658A) = {}  \
             excedente = {}  D_tot = {}",
            fila.instancia,
            fila.area,
            fila.columnas,
            fila.filas,
            fila.ancho_m,
            fila.alto_m,
            fila.n_pos,
            fila.objetivo,
            fila.excedente,
            fila.d_tot
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
