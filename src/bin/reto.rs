//! Binario del reto CONAFOR (Entregable 2).
//!
//! Uso: `reto <subcomando> [--datos DIR] [--resultados DIR]`
//!   datos   escribe `DIR/*.csv` con los datos del reto
//!   malla   |V|, D_tot y pares por anillo de 1 ha y de cada polígono; exporta W
//!   montecarlo  escenarios de existentes y resúmenes de C y costo [--instancias …|todas]
//!   paso3  ajuste al polígono real por escenario [--instancias …|todas] [--compra CSV]
//!   paso1  barrido de γ y patrón base [--instancias 1ha,map_27|todas] [--solo-gamma-star]

use std::path::Path;
use std::process::ExitCode;

use reforestation::competencia::matriz_w;
use reforestation::config::Config;
use reforestation::datos::{Datos, N_ESPECIES};
use reforestation::escenarios::{
    estadisticos, generar, percentil, resumir, Escenario, ParamsMc,
};
use reforestation::paso3::{correr_instancia, leer_compra, leer_patron, EntradaPaso3};
use reforestation::malla::Malla;
use reforestation::paso1::{barrido_gamma, mejor_fila, FilaPaso1, Instancia};
use reforestation::reportes::{
    escribir_alcance, escribir_anillos, escribir_barrido_gamma, escribir_composicion_paso1,
    escribir_convergencia, escribir_patron, escribir_resumen_gamma, escribir_resumen_malla,
    escribir_w, resumen_malla,
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

const OPCIONES: [&str; 6] =
    ["--datos", "--resultados", "--config", "--set", "--instancias", "--compra"];
/// Opciones sin valor.
const BANDERAS: [&str; 1] = ["--solo-gamma-star"];

/// Tras el subcomando sólo se aceptan pares `--opción valor` y banderas conocidas.
fn validar_opciones(args: &[String]) -> Result<(), String> {
    let mut resto = args.iter().skip(2);
    while let Some(nombre) = resto.next() {
        if BANDERAS.contains(&nombre.as_str()) {
            continue;
        }
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

/// Se lee sin validar, se aplican todos los --set y se valida una sola vez.
fn cargar_config(args: &[String]) -> Result<Config, Box<dyn std::error::Error>> {
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
    Ok(config)
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
            let config = cargar_config(args)?;
            malla(Path::new(&datos_dir), Path::new(&resultados_dir), &config)
        }
        Some("paso1") => {
            let config = cargar_config(args)?;
            let instancias = opcion(args, "--instancias", "1ha,map_27")?;
            let solo_star = args.iter().any(|a| a == "--solo-gamma-star");
            paso1(
                Path::new(&datos_dir),
                Path::new(&resultados_dir),
                &config,
                &instancias,
                solo_star,
            )
        }
        Some("montecarlo") => {
            let config = cargar_config(args)?;
            let instancias = opcion(args, "--instancias", "1ha,map_27")?;
            montecarlo(Path::new(&datos_dir), Path::new(&resultados_dir), &config, &instancias)
        }
        Some("paso3") => {
            let config = cargar_config(args)?;
            let instancias = opcion(args, "--instancias", "1ha,map_27")?;
            let compra = opcion(args, "--compra", &format!("{resultados_dir}/paso2_compra.csv"))?;
            paso3(
                Path::new(&datos_dir),
                Path::new(&resultados_dir),
                &config,
                &instancias,
                Path::new(&compra),
            )
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

/// "todas", "1ha" o "map_<id>" separados por comas, con su área.
fn resolver_instancias(
    datos: &Datos,
    instancias: &str,
) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
    let mut pedidas: Vec<(String, f64)> = Vec::new();
    for nombre in instancias.split(',').map(str::trim).filter(|n| !n.is_empty()) {
        if nombre == "todas" {
            pedidas.push(("1ha".to_string(), 1.0));
            pedidas.extend(datos.poligonos.iter().map(|p| (format!("map_{}", p.map_id), p.area)));
        } else if nombre == "1ha" {
            pedidas.push((nombre.to_string(), 1.0));
        } else {
            let poligono = nombre
                .strip_prefix("map_")
                .and_then(|m| m.parse::<u32>().ok())
                .and_then(|m| datos.poligono_por_map_id(m))
                .ok_or_else(|| format!("instancia desconocida: {nombre}"))?;
            pedidas.push((nombre.to_string(), poligono.area));
        }
    }
    if pedidas.is_empty() {
        return Err("--instancias no nombra ninguna instancia".into());
    }
    Ok(pedidas)
}

/// (D*, D nominal, L) de una instancia en `paso1_composicion.csv`.
type Composicion = ([u32; N_ESPECIES], [u32; N_ESPECIES], [u32; N_ESPECIES]);

fn leer_composicion(path: &Path, nombre: &str) -> Result<Composicion, Box<dyn std::error::Error>> {
    let mut lector = csv::Reader::from_path(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let encabezado = lector.headers()?.clone();
    for registro in lector.records() {
        let registro = registro?;
        if registro.get(0) != Some(nombre) {
            continue;
        }
        let tomar = |prefijo: &str| -> Result<[u32; N_ESPECIES], Box<dyn std::error::Error>> {
            let mut v = [0u32; N_ESPECIES];
            for (j, x) in v.iter_mut().enumerate() {
                let columna = format!("{prefijo}_{j}");
                let i = encabezado
                    .iter()
                    .position(|h| h == columna)
                    .ok_or_else(|| format!("falta la columna {columna}"))?;
                *x = registro[i].parse()?;
            }
            Ok(v)
        };
        return Ok((tomar("D")?, tomar("Dnom")?, tomar("L")?));
    }
    Err(format!("{}: no tiene la instancia {nombre}", path.display()).into())
}

fn escribir_escenarios(
    path: &Path,
    escenarios: &[Escenario],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut csv = csv::Writer::from_path(path)?;
    let mut encabezado = vec!["omega".to_string(), "T".to_string()];
    encabezado.extend((0..N_ESPECIES).map(|j| format!("X_{j}")));
    csv.write_record(&encabezado)?;
    for e in escenarios {
        let mut r = vec![e.omega.to_string(), e.t.to_string()];
        r.extend(e.x.iter().map(|x| x.to_string()));
        csv.write_record(&r)?;
    }
    csv.flush()?;
    Ok(())
}

fn montecarlo(
    datos_dir: &Path,
    resultados_dir: &Path,
    config: &Config,
    instancias: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let datos = Datos::cargar(datos_dir)?;
    let pedidas = resolver_instancias(&datos, instancias)?;
    let params = ParamsMc::leer(&resultados_dir.join("mc_params.json"))?;
    let composicion = resultados_dir.join("paso1_composicion.csv");
    let costos: [f64; N_ESPECIES] = std::array::from_fn(|j| datos.especies[j].c);
    let inicio = std::time::Instant::now();

    let mut esp = csv::Writer::from_path(resultados_dir.join("mc_resumen_especies.csv"))?;
    esp.write_record([
        "instancia", "map_id", "composicion", "especie", "D", "E_C", "sd_C", "ic95_inf",
        "ic95_sup", "P90", "P95", "N_necesario",
    ])?;
    let mut cos = csv::Writer::from_path(resultados_dir.join("mc_resumen_costo.csv"))?;
    cos.write_record([
        "instancia", "map_id", "composicion", "costo_esperado", "costo_sd", "presupuesto",
        "P_costo_mayor_B", "violaciones_X_mayor_L", "escenarios_C_negativo", "especies_C_negativo",
    ])?;
    for (nombre, area) in &pedidas {
        let map_id = nombre.strip_prefix("map_").and_then(|m| m.parse().ok()).unwrap_or(0u32);
        let (d_star, d_nom, l) = leer_composicion(&composicion, nombre)?;
        let d_tot = (datos.densidad_total() as f64 * area).round() as usize;
        let n_pos = Malla::para_objetivo(d_tot, config.d).len();
        let escenarios: Vec<_> = (0..config.n_escenarios)
            .map(|omega| generar(n_pos, *area, map_id, omega, config.semilla, &params))
            .collect();
        escribir_escenarios(&resultados_dir.join(format!("escenarios_X_{nombre}.csv")), &escenarios)?;
        for (etiqueta, d) in [("nominal", &d_nom), ("gamma_star", &d_star)] {
            let (filas, costo) = resumir(&escenarios, d, &l, &costos, config.presupuesto_ha * area);
            for f in &filas {
                esp.write_record([
                    nombre.clone(),
                    map_id.to_string(),
                    etiqueta.to_string(),
                    f.especie.to_string(),
                    f.d.to_string(),
                    f.e_c.to_string(),
                    f.sd_c.to_string(),
                    f.ic95_inf.to_string(),
                    f.ic95_sup.to_string(),
                    f.p90.to_string(),
                    f.p95.to_string(),
                    f.n_necesario.to_string(),
                ])?;
            }
            cos.write_record([
                nombre.clone(),
                map_id.to_string(),
                etiqueta.to_string(),
                costo.costo_esperado.to_string(),
                costo.costo_sd.to_string(),
                costo.presupuesto.to_string(),
                costo.p_costo_mayor_b.to_string(),
                costo.violaciones_x_mayor_l.to_string(),
                costo.escenarios_c_negativo.to_string(),
                costo.especies_c_negativo.to_string(),
            ])?;
        }
        println!("{nombre}: |V| = {n_pos}, {} escenarios", escenarios.len());
    }
    esp.flush()?;
    cos.flush()?;
    println!("montecarlo: {:.1} s", inicio.elapsed().as_secs_f64());
    Ok(())
}

fn paso1(
    datos_dir: &Path,
    resultados_dir: &Path,
    config: &Config,
    instancias: &str,
    solo_star: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let datos = Datos::cargar(datos_dir)?;
    std::fs::create_dir_all(resultados_dir)?;
    let pedidas = resolver_instancias(&datos, instancias)?;

    let mut gammas = if solo_star { vec![] } else { config.gammas.clone() };
    if !gammas.contains(&config.gamma_star) {
        gammas.push(config.gamma_star);
    }
    let semillas: Vec<u64> = (0..config.n_semillas as u64).map(|k| config.semilla + k).collect();
    let params = config.tabu.params(config.semilla);

    let inicio = std::time::Instant::now();
    let mut todas: Vec<FilaPaso1> = Vec::new();
    let mut elegidas: Vec<(Instancia, FilaPaso1)> = Vec::new();
    for (nombre, area) in &pedidas {
        let instancia =
            Instancia::nueva(nombre, *area, &datos, config, config.m_escala, config.usar_s2);
        let filas = barrido_gamma(&instancia, &gammas, &semillas, &params);
        let estrella = mejor_fila(filas.iter().filter(|f| f.gamma == config.gamma_star))
            .expect("γ* siempre se corre")
            .clone();
        println!(
            "{nombre}: D_tot = {}, γ* = {}: Z_opt = {:.2} (Z/planta {:.4}), D = {:?}",
            instancia.d_tot, config.gamma_star, estrella.z_opt, estrella.z_opt_por_planta, estrella.d
        );
        todas.extend(filas);
        elegidas.push((instancia, estrella));
    }

    if solo_star {
        // No pisa el barrido de γ ya publicado.
        escribir_barrido_gamma(&resultados_dir.join("paso1_gamma_star.csv"), &todas)?;
    } else {
        escribir_barrido_gamma(&resultados_dir.join("paso1_barrido_gamma.csv"), &todas)?;
        escribir_resumen_gamma(&resultados_dir.join("paso1_resumen_gamma.csv"), &todas)?;
    }
    let pares: Vec<(&Instancia, &FilaPaso1)> = elegidas.iter().map(|(i, f)| (i, f)).collect();
    escribir_composicion_paso1(&resultados_dir.join("paso1_composicion.csv"), &pares)?;
    for (instancia, fila) in &elegidas {
        escribir_patron(
            &resultados_dir.join(format!("paso1_patron_{}.csv", instancia.nombre)),
            &instancia.malla,
            &fila.phi,
        )?;
        escribir_convergencia(
            &resultados_dir.join(format!("convergencia_{}.csv", instancia.nombre)),
            fila,
        )?;
    }
    println!("paso 1: {:.1} s", inicio.elapsed().as_secs_f64());
    Ok(())
}

fn paso3(
    datos_dir: &Path,
    resultados_dir: &Path,
    config: &Config,
    instancias: &str,
    compra: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let datos = Datos::cargar(datos_dir)?;
    let pedidas = resolver_instancias(&datos, instancias)?;
    let mc = ParamsMc::leer(&resultados_dir.join("mc_params.json"))?;
    let params = config.tabu.params(config.semilla);
    let inicio = std::time::Instant::now();

    let mut esc = csv::Writer::from_path(resultados_dir.join("paso3_escenarios.csv"))?;
    esc.write_record([
        "instancia", "map_id", "omega", "caso", "T", "V_lib", "M", "Z", "Z_por_planta",
        "tiempo_s", "iteraciones",
    ])?;
    let mut res = csv::Writer::from_path(resultados_dir.join("paso3_resumen.csv"))?;
    res.write_record([
        "instancia", "map_id", "n", "Z_media", "Z_sd", "Z_p5", "Z_p50", "Z_p95", "casos_usual",
        "casos_presupuesto_corto", "casos_existentes_de_sobra",
    ])?;
    for (nombre, area) in &pedidas {
        let t0 = std::time::Instant::now();
        let instancia =
            Instancia::nueva(nombre, *area, &datos, config, config.m_escala, config.usar_s2);
        let base = leer_patron(&resultados_dir.join(format!("paso1_patron_{nombre}.csv")))?;
        let (d, _, _) = leer_composicion(&resultados_dir.join("paso1_composicion.csv"), nombre)?;
        let q = leer_compra(compra, nombre, config.epsilon, config.pi_factor)?;
        let entrada = EntradaPaso3 {
            instancia: &instancia,
            base: &base,
            q,
            d,
            mc: &mc,
            semilla: config.semilla,
        };
        let (filas, phi0, e0) = correr_instancia(&entrada, config.n_escenarios_paso3, &params);
        for f in &filas {
            esc.write_record([
                f.instancia.clone(),
                f.map_id.to_string(),
                f.omega.to_string(),
                f.caso.como_str().to_string(),
                f.t.to_string(),
                f.v_lib.to_string(),
                f.m.to_string(),
                f.z.to_string(),
                f.z_por_planta.to_string(),
                f.tiempo_s.to_string(),
                f.iteraciones.to_string(),
            ])?;
        }
        let z: Vec<f64> = filas.iter().map(|f| f.z).collect();
        let (media, sd) = estadisticos(&z);
        let casos = |c: reforestation::paso3::Caso| filas.iter().filter(|f| f.caso == c).count();
        use reforestation::paso3::Caso;
        res.write_record([
            nombre.clone(),
            instancia.map_id.to_string(),
            filas.len().to_string(),
            media.to_string(),
            sd.to_string(),
            percentil(&z, 0.05).to_string(),
            percentil(&z, 0.50).to_string(),
            percentil(&z, 0.95).to_string(),
            casos(Caso::Usual).to_string(),
            casos(Caso::PresupuestoCorto).to_string(),
            casos(Caso::ExistentesDeSobra).to_string(),
        ])?;
        // Patrón del escenario 0 con la marca de las existentes, para los mapas.
        let mut patron = csv::Writer::from_path(
            resultados_dir.join(format!("paso3_patron_{nombre}_omega0.csv")),
        )?;
        patron.write_record(["pos", "fila", "col", "x", "y", "especie", "existente"])?;
        let existentes: std::collections::HashSet<usize> =
            e0.posiciones.iter().map(|p| p.0).collect();
        for (u, p) in instancia.malla.posiciones.iter().enumerate() {
            patron.write_record([
                u.to_string(),
                p.fila.to_string(),
                p.col.to_string(),
                p.x.to_string(),
                p.y.to_string(),
                (phi0[u] as i32 - 1).to_string(),
                (existentes.contains(&u) as u8).to_string(),
            ])?;
        }
        patron.flush()?;
        println!(
            "{nombre}: Z media {media:.2} (sd {sd:.2}), usual/corto/sobra = {}/{}/{}, {:.1} s",
            casos(Caso::Usual),
            casos(Caso::PresupuestoCorto),
            casos(Caso::ExistentesDeSobra),
            t0.elapsed().as_secs_f64()
        );
    }
    esc.flush()?;
    res.flush()?;
    println!("paso 3: {:.1} s", inicio.elapsed().as_secs_f64());
    Ok(())
}
