//! Binario del reto CONAFOR (Entregable 2).
//!
//! Uso: `reto <subcomando> [--datos DIR] [--resultados DIR]`
//!   datos   escribe `DIR/*.csv` con los datos del reto
//!   malla   |V|, D_tot y pares por anillo de 1 ha y de cada polígono; exporta W
//!   montecarlo  escenarios de existentes y resúmenes de C y costo [--instancias …|todas]
//!   paso3  ajuste al polígono real por escenario [--instancias …|todas] [--compra CSV]
//!   practica  práctica actual (serpentina) contra el tabú [--instancias …|todas]
//!   calibrar  rejilla K x tenure del tabú [--instancias …] [--candidatos …] [--tenures …] [--semillas N]
//!   paso1  barrido de γ y patrón base [--instancias 1ha,map_27|todas] [--solo-gamma-star]
//!   instancias-val  escribe `validacion/inst_<id>.json` (10 instancias de validación exacta)
//!   tabu-val  tabú sobre cada `validacion/inst_*.json` → `tabu_<id>.csv` [--semillas 10]
//!   evaluar-val  z y objetivo de asignaciones: --inst JSON --phi CSV

use std::path::Path;
use std::process::ExitCode;

use reforestation::busqueda_tabu::MotivoParo;
use reforestation::calibracion::{calibrar, elegir};
use reforestation::competencia::matriz_w;
use reforestation::config::Config;
use reforestation::datos::{Datos, N_ESPECIES};
use reforestation::escenarios::{estadisticos, generar, percentil, resumir, Escenario, ParamsMc};
use reforestation::malla::Malla;
use reforestation::paso1::{barrido_gamma, mejor_fila, FilaPaso1, Instancia};
use reforestation::paso1::{patron_serpentina, reparar_a_banda};
use reforestation::paso3::{correr_instancia, leer_compra, leer_patron, EntradaPaso3};
use reforestation::practica::{desglose, serpentina_con_existentes};
use reforestation::reportes::{
    escribir_alcance, escribir_anillos, escribir_barrido_gamma, escribir_composicion_paso1,
    escribir_convergencia, escribir_patron, escribir_resumen_gamma, escribir_resumen_malla,
    escribir_w, resumen_malla,
};
use reforestation::tabu::DEFAULT_ROW_SEQUENCE;
use reforestation::validacion::{
    coincide, construir, correr_tabu, evaluar, leer_asignaciones, phi_a_texto, red, violaciones,
    InstanciaVal, ESPECIFICACIONES,
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

const OPCIONES: [&str; 11] = [
    "--datos",
    "--resultados",
    "--config",
    "--set",
    "--instancias",
    "--compra",
    "--candidatos",
    "--tenures",
    "--semillas",
    "--inst",
    "--phi",
];
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
            montecarlo(
                Path::new(&datos_dir),
                Path::new(&resultados_dir),
                &config,
                &instancias,
            )
        }
        Some("paso3") => {
            let config = cargar_config(args)?;
            let instancias = opcion(args, "--instancias", "1ha,map_27")?;
            let compra = opcion(
                args,
                "--compra",
                &format!("{resultados_dir}/paso2_compra.csv"),
            )?;
            paso3(
                Path::new(&datos_dir),
                Path::new(&resultados_dir),
                &config,
                &instancias,
                Path::new(&compra),
            )
        }
        Some("practica") => {
            let config = cargar_config(args)?;
            let instancias = opcion(args, "--instancias", "1ha,map_27")?;
            practica(
                Path::new(&datos_dir),
                Path::new(&resultados_dir),
                &config,
                &instancias,
            )
        }
        Some("calibrar") => {
            let config = cargar_config(args)?;
            let instancias = opcion(args, "--instancias", "1ha,map_21")?;
            let candidatos = lista_usize(&opcion(args, "--candidatos", "32,64,128")?)?;
            let tenures = lista_usize(&opcion(args, "--tenures", "10,15,25")?)?;
            let n_semillas: u64 = opcion(args, "--semillas", "5")?.parse()?;
            calibrar_cli(
                Path::new(&datos_dir),
                Path::new(&resultados_dir),
                &config,
                &instancias,
                &candidatos,
                &tenures,
                n_semillas,
            )
        }
        Some("instancias-val") => {
            let config = cargar_config(args)?;
            instancias_val(Path::new(&datos_dir), Path::new(&resultados_dir), &config)
        }
        Some("tabu-val") => {
            let config = cargar_config(args)?;
            let n_semillas: u64 = opcion(args, "--semillas", "10")?.parse()?;
            tabu_val(
                Path::new(&datos_dir),
                Path::new(&resultados_dir),
                &config,
                n_semillas,
            )
        }
        Some("evaluar-val") => {
            let inst = opcion(args, "--inst", "")?;
            let phi = opcion(args, "--phi", "")?;
            if inst.is_empty() || phi.is_empty() {
                return Err("evaluar-val requiere --inst JSON y --phi CSV".into());
            }
            evaluar_val(Path::new(&inst), Path::new(&phi))
        }
        otro => Err(format!("subcomando desconocido: {}", otro.unwrap_or("")).into()),
    }
}

fn lista_usize(texto: &str) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let valores = texto
        .split(',')
        .map(|v| v.trim().parse::<usize>())
        .collect::<Result<Vec<_>, _>>()?;
    if valores.is_empty() {
        return Err("lista vacía".into());
    }
    Ok(valores)
}

/// Rejilla K x tenure sobre cada instancia con γ*; escribe `calibracion_tabu.csv`.
fn calibrar_cli(
    datos_dir: &Path,
    resultados_dir: &Path,
    config: &Config,
    instancias: &str,
    candidatos: &[usize],
    tenures: &[usize],
    n_semillas: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let datos = Datos::cargar(datos_dir)?;
    std::fs::create_dir_all(resultados_dir)?;
    let pedidas = resolver_instancias(&datos, instancias)?;
    let semillas: Vec<u64> = (0..n_semillas).map(|k| config.semilla + k).collect();
    let mut csv = csv::Writer::from_path(resultados_dir.join("calibracion_tabu.csv"))?;
    csv.write_record([
        "instancia",
        "candidatos",
        "tenure",
        "n_semillas",
        "z_opt_media",
        "z_opt_sd",
        "z_opt_mejor",
        "objetivo_media",
        "tiempo_medio_s",
        "iter_media",
        "frac_sin_mejora",
    ])?;
    let mut todas = Vec::new();
    for (nombre, area) in &pedidas {
        let instancia = Instancia::nueva(
            nombre,
            *area,
            &datos,
            config,
            config.m_escala,
            config.usar_s2,
        );
        let filas = calibrar(
            &instancia,
            config.gamma_star,
            &config.tabu,
            candidatos,
            tenures,
            &semillas,
        );
        for f in &filas {
            csv.write_record([
                f.instancia.clone(),
                f.candidatos.to_string(),
                f.tenure.to_string(),
                f.n_semillas.to_string(),
                f.z_opt_media.to_string(),
                f.z_opt_sd.to_string(),
                f.z_opt_mejor.to_string(),
                f.objetivo_media.to_string(),
                f.tiempo_medio_s.to_string(),
                f.iter_media.to_string(),
                f.frac_sin_mejora.to_string(),
            ])?;
            println!(
                "{nombre}: K = {}, tenure = {}: Z_opt = {:.2} ± {:.2}, {:.1} s",
                f.candidatos, f.tenure, f.z_opt_media, f.z_opt_sd, f.tiempo_medio_s
            );
        }
        todas.extend(filas);
    }
    csv.flush()?;
    let (k, tenure) = elegir(&todas);
    println!("combinación elegida: K = {k}, tenure = {tenure}");
    Ok(())
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
    for nombre in instancias
        .split(',')
        .map(str::trim)
        .filter(|n| !n.is_empty())
    {
        if nombre == "todas" {
            pedidas.push(("1ha".to_string(), 1.0));
            pedidas.extend(
                datos
                    .poligonos
                    .iter()
                    .map(|p| (format!("map_{}", p.map_id), p.area)),
            );
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
    let mut lector =
        csv::Reader::from_path(path).map_err(|e| format!("{}: {e}", path.display()))?;
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
        "instancia",
        "map_id",
        "composicion",
        "especie",
        "D",
        "E_C",
        "sd_C",
        "ic95_inf",
        "ic95_sup",
        "P90",
        "P95",
        "N_necesario",
    ])?;
    let mut cos = csv::Writer::from_path(resultados_dir.join("mc_resumen_costo.csv"))?;
    cos.write_record([
        "instancia",
        "map_id",
        "composicion",
        "costo_esperado",
        "costo_sd",
        "presupuesto",
        "P_costo_mayor_B",
        "violaciones_X_mayor_L",
        "escenarios_C_negativo",
        "especies_C_negativo",
    ])?;
    for (nombre, area) in &pedidas {
        let map_id = nombre
            .strip_prefix("map_")
            .and_then(|m| m.parse().ok())
            .unwrap_or(0u32);
        let (d_star, d_nom, l) = leer_composicion(&composicion, nombre)?;
        let d_tot = (datos.densidad_total() as f64 * area).round() as usize;
        let n_pos = Malla::para_objetivo(d_tot, config.d).len();
        let escenarios: Vec<_> = (0..config.n_escenarios)
            .map(|omega| generar(n_pos, *area, map_id, omega, config.semilla, &params))
            .collect();
        escribir_escenarios(
            &resultados_dir.join(format!("escenarios_X_{nombre}.csv")),
            &escenarios,
        )?;
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

    let mut gammas = if solo_star {
        vec![]
    } else {
        config.gammas.clone()
    };
    if !gammas.contains(&config.gamma_star) {
        gammas.push(config.gamma_star);
    }
    let semillas: Vec<u64> = (0..config.n_semillas as u64)
        .map(|k| config.semilla + k)
        .collect();

    let inicio = std::time::Instant::now();
    let mut todas: Vec<FilaPaso1> = Vec::new();
    let mut elegidas: Vec<(Instancia, FilaPaso1)> = Vec::new();
    for (nombre, area) in &pedidas {
        let instancia = Instancia::nueva(
            nombre,
            *area,
            &datos,
            config,
            config.m_escala,
            config.usar_s2,
        );
        let params = config
            .tabu
            .params_para(config.semilla, instancia.malla.len());
        let filas = barrido_gamma(&instancia, &gammas, &semillas, &params);
        let estrella = mejor_fila(filas.iter().filter(|f| f.gamma == config.gamma_star))
            .expect("γ* siempre se corre")
            .clone();
        println!(
            "{nombre}: D_tot = {}, γ* = {}: Z_opt = {:.2} (Z/planta {:.4}), D = {:?}",
            instancia.d_tot,
            config.gamma_star,
            estrella.z_opt,
            estrella.z_opt_por_planta,
            estrella.d
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
    let inicio = std::time::Instant::now();

    let mut esc = csv::Writer::from_path(resultados_dir.join("paso3_escenarios.csv"))?;
    esc.write_record([
        "instancia",
        "map_id",
        "omega",
        "caso",
        "T",
        "V_lib",
        "M",
        "Z",
        "Z_por_planta",
        "tiempo_s",
        "iteraciones",
        "motivo_paro",
    ])?;
    let mut res = csv::Writer::from_path(resultados_dir.join("paso3_resumen.csv"))?;
    res.write_record([
        "instancia",
        "map_id",
        "n",
        "Z_media",
        "Z_sd",
        "Z_p5",
        "Z_p50",
        "Z_p95",
        "casos_usual",
        "casos_presupuesto_corto",
        "casos_existentes_de_sobra",
        "paros_tope",
        "paros_sin_mejora",
    ])?;
    for (nombre, area) in &pedidas {
        let t0 = std::time::Instant::now();
        let instancia = Instancia::nueva(
            nombre,
            *area,
            &datos,
            config,
            config.m_escala,
            config.usar_s2,
        );
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
        let params = config
            .tabu
            .params_para(config.semilla, instancia.malla.len());
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
                f.motivo_paro.como_str().to_string(),
            ])?;
        }
        let z: Vec<f64> = filas.iter().map(|f| f.z).collect();
        let (media, sd) = estadisticos(&z);
        let casos = |c: reforestation::paso3::Caso| filas.iter().filter(|f| f.caso == c).count();
        use reforestation::paso3::Caso;
        let paros = |m: MotivoParo| filas.iter().filter(|f| f.motivo_paro == m).count();
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
            paros(MotivoParo::Tope).to_string(),
            paros(MotivoParo::SinMejora).to_string(),
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

/// M (plantas nuevas) del escenario ω = 0 de `paso3_escenarios.csv`.
fn leer_m_omega0(path: &Path, nombre: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let mut lector =
        csv::Reader::from_path(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let enc = lector.headers()?.clone();
    let col = |n: &str| {
        enc.iter()
            .position(|h| h == n)
            .ok_or_else(|| format!("falta {n}"))
    };
    let (ci, co, cm) = (col("instancia")?, col("omega")?, col("M")?);
    for r in lector.records() {
        let r = r?;
        if &r[ci] == nombre && &r[co] == "0" {
            return Ok(r[cm].parse()?);
        }
    }
    Err(format!("{}: sin ω = 0 para {nombre}", path.display()).into())
}

/// Valor de la columna `columna` de la primera fila que cumple `filtro`; vacío si falta el
/// archivo, la columna o la fila (CSV de corridas anteriores a la columna de motivo de paro).
fn primer_valor(
    path: &Path,
    columna: &str,
    filtro: impl Fn(&dyn Fn(&str) -> Option<String>) -> bool,
) -> String {
    let Ok(mut lector) = csv::Reader::from_path(path) else {
        return String::new();
    };
    let Ok(enc) = lector.headers().cloned() else {
        return String::new();
    };
    for r in lector.records().flatten() {
        let campo = |n: &str| enc.iter().position(|h| h == n).map(|i| r[i].to_string());
        if filtro(&campo) {
            return campo(columna).unwrap_or_default();
        }
    }
    String::new()
}

/// Motivo de paro del ajuste C de la semilla elegida en γ* (la de menor objetivo del paso 1,
/// γ·z_paso1 + Σθ·D), leído de `paso1_gamma_star.csv` o, si no existe, del barrido.
fn leer_motivo_paso1(resultados_dir: &Path, nombre: &str, gamma_star: f64) -> String {
    for archivo in ["paso1_gamma_star.csv", "paso1_barrido_gamma.csv"] {
        let path = resultados_dir.join(archivo);
        let Ok(mut lector) = csv::Reader::from_path(&path) else {
            continue;
        };
        let Ok(enc) = lector.headers().cloned() else {
            continue;
        };
        let num = |r: &csv::StringRecord, n: &str| {
            enc.iter()
                .position(|h| h == n)
                .and_then(|i| r[i].parse::<f64>().ok())
        };
        let mut mejor: Option<(f64, String)> = None;
        for r in lector.records().flatten() {
            let (Some(g), Some(z), Some(t)) = (
                num(&r, "gamma"),
                num(&r, "z_paso1"),
                num(&r, "suma_theta_d"),
            ) else {
                continue;
            };
            let Some(i) = enc.iter().position(|h| h == "instancia") else {
                continue;
            };
            let Some(m) = enc.iter().position(|h| h == "motivo_paro_opt") else {
                continue;
            };
            if &r[i] != nombre || (g - gamma_star).abs() > 1e-9 {
                continue;
            }
            let objetivo = g * z + t;
            if mejor.as_ref().is_none_or(|(o, _)| objetivo < *o) {
                mejor = Some((objetivo, r[m].to_string()));
            }
        }
        if let Some((_, motivo)) = mejor {
            return motivo;
        }
    }
    String::new()
}

/// Motivo de paro del escenario ω = 0 de `paso3_escenarios.csv`.
fn leer_motivo_paso3_omega0(resultados_dir: &Path, nombre: &str) -> String {
    primer_valor(
        &resultados_dir.join("paso3_escenarios.csv"),
        "motivo_paro",
        |c| c("instancia").as_deref() == Some(nombre) && c("omega").as_deref() == Some("0"),
    )
}

fn practica(
    datos_dir: &Path,
    resultados_dir: &Path,
    config: &Config,
    instancias: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let datos = Datos::cargar(datos_dir)?;
    let pedidas = resolver_instancias(&datos, instancias)?;
    let mc = ParamsMc::leer(&resultados_dir.join("mc_params.json"))?;
    let w = matriz_w(&datos.especies, config.kappa);
    let inicio = std::time::Instant::now();
    let mut csv = csv::Writer::from_path(resultados_dir.join("practica_distribucion.csv"))?;
    csv.write_record([
        "instancia",
        "variante",
        "plantas",
        "Z",
        "Z_por_planta",
        "pares_inmediatos_misma_especie",
        "Z_nivel1",
        "Z_nivel2",
        "Z_nivel3",
        "Z_nivel4",
        "Z_nivel5",
        "Z_otro",
        "motivo_paro",
    ])?;
    for (nombre, area) in &pedidas {
        let inst = Instancia::nueva(
            nombre,
            *area,
            &datos,
            config,
            config.m_escala,
            config.usar_s2,
        );
        let e0 = generar(
            inst.malla.len(),
            inst.area,
            inst.map_id,
            0,
            config.semilla,
            &mc,
        );
        let m = leer_m_omega0(&resultados_dir.join("paso3_escenarios.csv"), nombre)?;
        let i = patron_serpentina(&inst.malla, DEFAULT_ROW_SEQUENCE, inst.d_tot);
        let mut ii = i.clone();
        reparar_a_banda(&mut ii, &inst.banda);
        let iii = serpentina_con_existentes(&inst.malla, &e0.posiciones, DEFAULT_ROW_SEQUENCE, m);
        let sin = leer_patron(&resultados_dir.join(format!("paso1_patron_{nombre}.csv")))?;
        let con = leer_patron(&resultados_dir.join(format!("paso3_patron_{nombre}_omega0.csv")))?;
        // Motivo de paro de las dos variantes tabú, tomado de los CSV de los pasos 1 y 3.
        let motivo_sin = leer_motivo_paso1(resultados_dir, nombre, config.gamma_star);
        let motivo_con = leer_motivo_paso3_omega0(resultados_dir, nombre);
        let variantes = [
            ("i_serpentina", &i, String::new()),
            ("ii_serpentina_banda", &ii, String::new()),
            ("iii_serpentina_existentes", &iii, String::new()),
            ("tabu_sin_existentes", &sin, motivo_sin),
            ("tabu_con_existentes", &con, motivo_con),
        ];
        for (variante, phi, motivo) in variantes {
            let d = desglose(phi, &inst.grafo, &inst.tabla, &datos.especies, &w);
            let mut fila = vec![
                nombre.clone(),
                variante.to_string(),
                d.plantas.to_string(),
                d.z.to_string(),
                (if d.plantas > 0 {
                    d.z / d.plantas as f64
                } else {
                    0.0
                })
                .to_string(),
                d.pares_inmediatos.to_string(),
            ];
            fila.extend(d.niveles.iter().map(|x| x.to_string()));
            fila.push(motivo);
            csv.write_record(&fila)?;
        }
        if nombre == "1ha" || nombre == "map_27" {
            let mut patron = csv::Writer::from_path(
                resultados_dir.join(format!("practica_patron_{nombre}_iii.csv")),
            )?;
            patron.write_record(["pos", "fila", "col", "x", "y", "especie", "existente"])?;
            let existentes: std::collections::HashSet<usize> =
                e0.posiciones.iter().map(|p| p.0).collect();
            for (u, p) in inst.malla.posiciones.iter().enumerate() {
                patron.write_record([
                    u.to_string(),
                    p.fila.to_string(),
                    p.col.to_string(),
                    p.x.to_string(),
                    p.y.to_string(),
                    (iii[u] as i32 - 1).to_string(),
                    (existentes.contains(&u) as u8).to_string(),
                ])?;
            }
            patron.flush()?;
        }
    }
    csv.flush()?;
    println!("practica: {:.1} s", inicio.elapsed().as_secs_f64());
    Ok(())
}

/// Escribe las 10 instancias de validación en `resultados/validacion/inst_<id>.json`.
fn instancias_val(
    datos_dir: &Path,
    resultados_dir: &Path,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let datos = Datos::cargar(datos_dir)?;
    let mc = ParamsMc::leer(&resultados_dir.join("mc_params.json"))?;
    let dir = resultados_dir.join("validacion");
    std::fs::create_dir_all(&dir)?;
    for e in &ESPECIFICACIONES {
        let inst = construir(e, &datos, config, &mc);
        std::fs::write(
            dir.join(format!("inst_{}.json", inst.id)),
            serde_json::to_string(&inst)?,
        )?;
        let fijas = inst.fija.iter().filter(|&&f| f >= 0).count();
        println!(
            "{}: |V| = {}, fijas = {fijas}, M = {:?}, D_tot = {:?}, aristas = {}",
            inst.id,
            inst.n_pos,
            inst.m,
            inst.d_tot,
            inst.aristas.len()
        );
    }
    Ok(())
}

/// Tabú con semillas `semilla + k` sobre cada `validacion/inst_*.json` → `tabu_<id>.csv`.
fn tabu_val(
    datos_dir: &Path,
    resultados_dir: &Path,
    config: &Config,
    n_semillas: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let datos = Datos::cargar(datos_dir)?;
    let dir = resultados_dir.join("validacion");
    let mut archivos: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("inst_") && n.ends_with(".json"))
        })
        .collect();
    archivos.sort();
    if archivos.is_empty() {
        return Err(format!("{}: sin inst_*.json (correr instancias-val)", dir.display()).into());
    }
    for path in archivos {
        let inst: InstanciaVal = serde_json::from_str(&std::fs::read_to_string(&path)?)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if inst.disposicion != "Completa" {
            return Err(format!("{}: sólo se admiten mallas Completa", path.display()).into());
        }
        let r = red(inst.columnas, inst.filas, &datos, config);
        if !coincide(&inst, &r) {
            return Err(format!(
                "{}: aristas o pesos distintos a los de la configuración actual",
                path.display()
            )
            .into());
        }
        let mut csv = csv::Writer::from_path(dir.join(format!("tabu_{}.csv", inst.id)))?;
        csv.write_record([
            "semilla",
            "z",
            "objetivo",
            "tiempo_s",
            "iteraciones",
            "motivo_paro",
            "phi",
        ])?;
        let mut zs = Vec::new();
        for k in 0..n_semillas {
            let c = correr_tabu(&inst, &r, &config.tabu, config.semilla + k);
            let fallas = violaciones(&inst, &c.phi);
            if !fallas.is_empty() {
                return Err(format!("{} semilla {}: {fallas:?}", inst.id, c.semilla).into());
            }
            csv.write_record([
                c.semilla.to_string(),
                c.z.to_string(),
                c.objetivo.to_string(),
                c.tiempo_s.to_string(),
                c.iteraciones.to_string(),
                c.motivo_paro.como_str().to_string(),
                phi_a_texto(&c.phi),
            ])?;
            zs.push(c.z);
        }
        csv.flush()?;
        let mejor = zs.iter().copied().fold(f64::INFINITY, f64::min);
        let (media, _) = estadisticos(&zs);
        println!("{}: z mejor {mejor:.6}, media {media:.6}", inst.id);
    }
    Ok(())
}

/// Imprime `z,objetivo` (12 decimales) de cada asignación del CSV; avisa en stderr si alguna
/// viola las restricciones de la instancia.
fn evaluar_val(inst: &Path, phi: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let inst: InstanciaVal = serde_json::from_str(
        &std::fs::read_to_string(inst).map_err(|e| format!("{}: {e}", inst.display()))?,
    )
    .map_err(|e| format!("{}: {e}", inst.display()))?;
    let asignaciones = leer_asignaciones(phi)?;
    println!("z,objetivo");
    for (k, phi) in asignaciones.iter().enumerate() {
        if phi.len() != inst.n_pos {
            return Err(format!(
                "asignación {k}: {} posiciones, la instancia tiene {}",
                phi.len(),
                inst.n_pos
            )
            .into());
        }
        for falla in violaciones(&inst, phi) {
            eprintln!("aviso: asignación {k}: {falla}");
        }
        let (z, objetivo) = evaluar(&inst, phi);
        println!("{z:.12},{objetivo:.12}");
    }
    Ok(())
}
