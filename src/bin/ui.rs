use std::path::Path;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use eframe::egui;
use egui::{Color32, Pos2, Sense, Shape, Stroke, Vec2};

use reforestation::busqueda_tabu::ParamsTabu;
use reforestation::composicion::Banda;
use reforestation::config::Config;
use reforestation::datos::{Datos, N_ESPECIES};
use reforestation::escenarios::{estadisticos, percentil, ParamsMc};
use reforestation::malla::Posicion;
use reforestation::objetivo::z_total;
use reforestation::paso1::{patron_serpentina, reparar_a_banda, resolver_paso1, Instancia};
use reforestation::paso3::{self, Caso, EntradaPaso3, FilaPaso3};
use reforestation::writer::{read_plants_file, read_polygons_file};
use reforestation::{Plant, Polygon};

const PALETTE: [Color32; 10] = [
    Color32::from_rgb(228, 26, 28),
    Color32::from_rgb(55, 126, 184),
    Color32::from_rgb(77, 175, 74),
    Color32::from_rgb(152, 78, 163),
    Color32::from_rgb(255, 127, 0),
    Color32::from_rgb(255, 214, 51),
    Color32::from_rgb(166, 86, 40),
    Color32::from_rgb(247, 129, 191),
    Color32::from_rgb(0, 200, 200),
    Color32::from_rgb(170, 170, 170),
];

use reforestation::tabu::DEFAULT_ROW_SEQUENCE as ROW_SEQUENCE;

fn species_color(species: usize) -> Color32 {
    PALETTE[species % PALETTE.len()]
}

/// p̂ de `results/mc_params.json` (respaldo si el archivo no existe).
const MC_RESPALDO: ParamsMc = ParamsMc {
    mu: 129.182_340_403_955_83,
    sigma: 3.289_562_611_809_45,
    p_hat: [
        0.063_261_063_261_063_26,
        0.295_133_437_990_580_84,
        0.062_030_633_459_204_9,
        0.059_569_773_855_488_14,
        0.077_729_220_586_363_45,
        0.061_903_347_617_633_33,
        0.116_508_973_651_830_8,
        0.097_119_097_119_097_12,
        0.128_049_556_620_985_2,
        0.038_694_895_837_752_98,
    ],
};

/// Qué malla se está mirando: el patrón de la banda (antes/después) o el paso 3.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Vista {
    Antes,
    Despues,
    Paso3,
}

/// Resultado del paso 1 (nuevo algoritmo, sin existentes fijas) para la UI.
#[derive(Clone)]
struct Paso1View {
    nombre: String,
    area: f64,
    filas: usize,
    columnas: usize,
    /// φ inicial: serpentina + reparación a banda (el punto de partida del tabú).
    phi_inicial: Vec<u8>,
    /// φ final (paso 1 + ajuste C). Codificación: 0 = vacía, especie j ↔ j + 1.
    phi: Vec<u8>,
    /// D* = conteo de φ.
    d: [u32; N_ESPECIES],
    /// D del patrón inicial.
    d_inicial: [u32; N_ESPECIES],
    /// Composición nominal D̄.
    nominal: Vec<usize>,
    banda: Banda,
    posiciones: Vec<Posicion>,
    /// Distancia entre vecinos inmediatos (m).
    separacion: f64,
    /// Z del patrón inicial.
    z_inicial: f64,
    /// Z* (tras el ajuste C).
    z: f64,
    z_por_planta: f64,
    /// Σ θ_j·D_j (costo ecológico de mortalidad).
    mortalidad: f64,
    /// Σ θ_j·D_j del patrón inicial.
    mortalidad_inicial: f64,
    gamma: f64,
    /// Objetivo γ·Z + Σθ·D del paso 1 en la mejor solución.
    objetivo: f64,
    /// Σ c_j·D_j (costo monetario de la composición).
    costo: f64,
    costo_ha: f64,
    costo_inicial: f64,
    presupuesto_ha: f64,
    iteraciones: usize,
    tiempo_s: f64,
}

/// Resultado del paso 3 (escenario 0 + resumen sobre todos los escenarios) para la UI.
#[derive(Clone)]
struct Paso3View {
    indice: usize,
    nombre: String,
    area: f64,
    /// D* (de `paso1_composicion.csv` o el nominal de la instancia).
    d: [u32; N_ESPECIES],
    /// q: cantidades compradas (paso 2 o demanda neta).
    q: [u32; N_ESPECIES],
    /// X_j del escenario 0.
    x: [u32; N_ESPECIES],
    /// n_j = min(q_j, C⁺_j) del escenario 0.
    n: Vec<usize>,
    banda: Banda,
    /// φ final: 0 = vacía, especie j ↔ j + 1.
    phi: Vec<u8>,
    existentes: Vec<bool>,
    posiciones: Vec<Posicion>,
    /// Distancia entre vecinos inmediatos (m).
    separacion: f64,
    fila: FilaPaso3,
    z_media: f64,
    z_sd: f64,
    z_p50: f64,
    z_p95: f64,
    /// [usual, presupuesto_corto, existentes_de_sobra].
    casos: [usize; 3],
    n_omega: usize,
    costo_compra: f64,
    costo_ha: f64,
    presupuesto_ha: f64,
    fuente_base: &'static str,
    fuente_q: &'static str,
}

/// D* de una instancia en `results/paso1_composicion.csv`.
fn leer_d_paso1(path: &Path, nombre: &str) -> Result<[u32; N_ESPECIES], String> {
    let mut lector =
        csv::Reader::from_path(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let enc = lector.headers().map_err(|e| e.to_string())?.clone();
    let columna = |nombre_col: &str| {
        enc.iter()
            .position(|h| h == nombre_col)
            .ok_or_else(|| format!("falta la columna {nombre_col}"))
    };
    let ci = columna("instancia")?;
    let columnas: Vec<usize> = (0..N_ESPECIES)
        .map(|j| columna(&format!("D_{j}")))
        .collect::<Result<_, String>>()?;
    for registro in lector.records() {
        let r = registro.map_err(|e| e.to_string())?;
        if &r[ci] == nombre {
            let mut d = [0u32; N_ESPECIES];
            for j in 0..N_ESPECIES {
                d[j] = r[columnas[j]]
                    .parse::<f64>()
                    .map_err(|e| e.to_string())?
                    .round() as u32;
            }
            return Ok(d);
        }
    }
    Err(format!("{nombre}: sin fila en {}", path.display()))
}

/// Corre el paso 3 completo para una instancia (hilo de trabajo; no toca la UI).
#[allow(clippy::too_many_arguments)]
fn correr_paso3(
    indice: usize,
    nombre: String,
    area: f64,
    datos: &Datos,
    config: &Config,
    n_omega: usize,
    existentes_reales: Option<[u32; N_ESPECIES]>,
) -> Result<Paso3View, String> {
    let params: ParamsTabu = config.tabu.params(config.semilla);
    let instancia = Instancia::nueva(
        &nombre,
        area,
        datos,
        config,
        config.m_escala,
        config.usar_s2,
    );
    let resultados = Path::new("results");

    // φ* del paso 1: del CSV publicado; si no existe, se resuelve aquí.
    let ruta_patron = resultados.join(format!("paso1_patron_{nombre}.csv"));
    let (base, fuente_base) = match paso3::leer_patron(&ruta_patron) {
        Ok(base) if base.len() == instancia.malla.len() => (base, "paso 1 CSV"),
        _ => (
            resolver_paso1(&instancia, config.gamma_star, &params).1,
            "tabú paso 1",
        ),
    };

    // D* del paso 1 o, en su falta, la composición nominal.
    let d = leer_d_paso1(&resultados.join("paso1_composicion.csv"), &nombre).unwrap_or_else(|_| {
        std::array::from_fn(|j| instancia.nominal[j].min(u32::MAX as usize) as u32)
    });

    // q del paso 2 (PLE) o, en su falta, la demanda neta contra los existentes reales.
    let compra = resultados.join("paso2_compra.csv");
    let (q, fuente_q) = match paso3::leer_compra(&compra, &nombre, config.epsilon, config.pi_factor)
    {
        Ok(q) => (q, "paso 2 (PLE)"),
        Err(_) => {
            let x0 = existentes_reales.unwrap_or([0; N_ESPECIES]);
            (
                std::array::from_fn(|j| d[j].saturating_sub(x0[j])),
                "demanda neta",
            )
        }
    };

    let mc = ParamsMc::leer(&resultados.join("mc_params.json")).unwrap_or(MC_RESPALDO);
    let entrada = EntradaPaso3 {
        instancia: &instancia,
        base: &base,
        q,
        d,
        mc: &mc,
        semilla: config.semilla,
    };
    let (filas, phi0, e0) = paso3::correr_instancia(&entrada, n_omega.max(1), &params);
    let fila = filas[0].clone();
    let (n, _, _) = paso3::cuotas(&q, &d, &e0.x, fila.v_lib);

    let mut existentes = vec![false; phi0.len()];
    for &(u, _) in &e0.posiciones {
        existentes[u] = true;
    }

    let z: Vec<f64> = filas.iter().map(|f| f.z).collect();
    let (z_media, z_sd) = estadisticos(&z);
    let contar = |c: Caso| filas.iter().filter(|f| f.caso == c).count();
    let casos = [
        contar(Caso::Usual),
        contar(Caso::PresupuestoCorto),
        contar(Caso::ExistentesDeSobra),
    ];

    let costo_compra: f64 = (0..N_ESPECIES)
        .map(|j| datos.especies[j].c * q[j] as f64)
        .sum();

    Ok(Paso3View {
        indice,
        nombre,
        area,
        d,
        q,
        x: e0.x,
        n,
        banda: instancia.banda.clone(),
        phi: phi0,
        existentes,
        posiciones: instancia.malla.posiciones.clone(),
        separacion: instancia.malla.d,
        fila,
        z_media,
        z_sd,
        z_p50: percentil(&z, 0.50),
        z_p95: percentil(&z, 0.95),
        casos,
        n_omega: filas.len(),
        costo_compra,
        costo_ha: if area > 0.0 { costo_compra / area } else { 0.0 },
        presupuesto_ha: config.presupuesto_ha,
        fuente_base,
        fuente_q,
    })
}

fn contar_por_especie(phi: &[u8]) -> [u32; N_ESPECIES] {
    let mut d = [0u32; N_ESPECIES];
    for &code in phi {
        if code > 0 {
            d[code as usize - 1] += 1;
        }
    }
    d
}

/// Corre el paso 1 (nuevo algoritmo, sin existentes fijas) para una instancia (hilo de trabajo).
fn correr_paso1(
    nombre: String,
    area: f64,
    datos: &Datos,
    config: &Config,
) -> Result<Paso1View, String> {
    let params: ParamsTabu = config.tabu.params(config.semilla);
    let instancia = Instancia::nueva(
        &nombre,
        area,
        datos,
        config,
        config.m_escala,
        config.usar_s2,
    );

    let mut phi_inicial = patron_serpentina(&instancia.malla, ROW_SEQUENCE, instancia.d_tot);
    reparar_a_banda(&mut phi_inicial, &instancia.banda);
    let (fila, phi) = resolver_paso1(&instancia, config.gamma_star, &params);

    let d = contar_por_especie(&phi);
    let d_inicial = contar_por_especie(&phi_inicial);
    let suma = |v: &[f64; N_ESPECIES], c: &[u32; N_ESPECIES]| {
        (0..N_ESPECIES).map(|j| v[j] * c[j] as f64).sum::<f64>()
    };
    let costo = suma(&instancia.costos, &d);
    let costo_inicial = suma(&instancia.costos, &d_inicial);
    let z_inicial = z_total(&phi_inicial, &instancia.grafo, &instancia.tabla);

    Ok(Paso1View {
        nombre,
        area,
        filas: instancia.malla.filas(),
        columnas: instancia.malla.columnas,
        phi_inicial,
        phi,
        d,
        d_inicial,
        nominal: instancia.nominal.clone(),
        banda: instancia.banda.clone(),
        posiciones: instancia.malla.posiciones.clone(),
        separacion: instancia.malla.d,
        z_inicial,
        z: fila.z_opt,
        z_por_planta: fila.z_opt_por_planta,
        mortalidad: fila.suma_theta_d,
        mortalidad_inicial: suma(&instancia.theta, &d_inicial),
        gamma: fila.gamma,
        objetivo: fila.objetivo_paso1,
        costo,
        costo_ha: if area > 0.0 { costo / area } else { 0.0 },
        costo_inicial,
        presupuesto_ha: config.presupuesto_ha,
        iteraciones: fila.iter_paso1 + fila.iter_opt,
        tiempo_s: fila.tiempo_paso1_s + fila.tiempo_opt_s,
    })
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 860.0])
            .with_title("Reforestation - Tabu Search"),
        ..Default::default()
    };

    eframe::run_native(
        "Reforestation",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}

struct App {
    plants: Vec<Plant>,
    polygons: Vec<Polygon>,
    resultados: Vec<Option<Paso1View>>,
    selected: usize,
    visible: Vec<bool>,
    vista: Vista,
    hovered: Option<usize>,
    inspected: Option<usize>,
    receiver: Option<Receiver<(usize, Result<Paso1View, String>)>>,
    pending: usize,
    status: String,
    custom_width: usize,
    custom_height: usize,
    datos: Option<Datos>,
    reto_config: Option<Config>,
    n_omega: usize,
    paso3: Option<Paso3View>,
    paso3_status: String,
    paso3_receiver: Option<Receiver<Result<Paso3View, String>>>,
}

impl App {
    fn new() -> Self {
        let plants = read_plants_file("bins/plants.bin");
        let polygons = read_polygons_file("bins/polygons.bin");
        let visible = vec![true; plants.len()];
        let entries = polygons.len() + 1;
        let resultados = vec![None; entries];

        let datos = Datos::cargar(Path::new("data")).ok();
        let reto_config = Config::cargar(Path::new("config/default.json")).ok();
        let n_omega = reto_config
            .as_ref()
            .map_or(10, |c| c.n_escenarios_paso3.clamp(1, 10));

        let mut app = App {
            plants,
            polygons,
            resultados,
            selected: 0,
            visible,
            vista: Vista::Despues,
            hovered: None,
            inspected: None,
            receiver: None,
            pending: 0,
            status: "Press \"Run selected\" to optimize the highlighted polygon.".to_string(),
            custom_width: 30,
            custom_height: 30,
            datos,
            reto_config,
            n_omega,
            paso3: None,
            paso3_status: "Pick a polygon and press \"Run paso 3\".".to_string(),
            paso3_receiver: None,
        };
        app.run_selected();
        app
    }

    fn entry_count(&self) -> usize {
        self.polygons.len() + 1
    }

    fn is_custom(&self, index: usize) -> bool {
        index >= self.polygons.len()
    }

    fn custom_density(&self) -> f64 {
        self.plants
            .iter()
            .map(|plant| plant.get_per_hectare() as f64)
            .sum()
    }

    fn custom_area(&self) -> f64 {
        let density = self.custom_density();
        if density > 0.0 {
            (self.custom_width * self.custom_height) as f64 / density
        } else {
            0.0
        }
    }

    /// Nombre de instancia (map_N o "custom") y área para el modelo nuevo; None si no hay datos.
    fn entrada_instancia(&self, index: usize) -> Option<(String, f64)> {
        if self.is_custom(index) {
            Some(("custom".to_string(), self.custom_area()))
        } else {
            self.datos
                .as_ref()?
                .poligonos
                .get(index)
                .map(|p| (format!("map_{}", p.map_id), p.area))
        }
    }

    /// Lanza el paso 3 (escenarios + búsqueda tabú con cuotas) en un hilo.
    fn run_paso3(&mut self) {
        if self.paso3_receiver.is_some() {
            return;
        }
        let Some(datos) = self.datos.clone() else {
            self.paso3_status =
                "data/ not found: run `reto datos` first (step 7 needs the CONAFOR data)."
                    .to_string();
            return;
        };
        let Some(config) = self.reto_config.clone() else {
            self.paso3_status =
                "config/default.json not found: paso 3 needs the model configuration.".to_string();
            return;
        };
        let Some((nombre, area)) = self.entrada_instancia(self.selected) else {
            self.paso3_status =
                "data/poligonos.csv does not match the polygons of bins/polygons.bin.".to_string();
            return;
        };
        let existentes = self
            .datos
            .as_ref()
            .and_then(|d| d.poligonos.get(self.selected))
            .map(|p| p.existentes);

        let indice = self.selected;
        let n_omega = self.n_omega;
        let (sender, receiver) = mpsc::channel();
        self.paso3_receiver = Some(receiver);
        self.paso3_status =
            format!("Solving {nombre}: {n_omega} escenarios (montecarlo + tabú)...");

        thread::spawn(move || {
            let resultado =
                correr_paso3(indice, nombre, area, &datos, &config, n_omega, existentes);
            let _ = sender.send(resultado);
        });
    }

    fn poll_paso3(&mut self, ctx: &egui::Context) {
        let Some(receiver) = self.paso3_receiver.take() else {
            return;
        };
        match receiver.try_recv() {
            Ok(Ok(view)) => {
                self.paso3_status = format!(
                    "{}: caso {}, Z {:.2} | {} escenarios, Z media {:.2}",
                    view.nombre,
                    view.fila.caso.como_str(),
                    view.fila.z,
                    view.n_omega,
                    view.z_media
                );
                self.paso3 = Some(view);
                self.vista = Vista::Paso3;
                self.hovered = None;
                self.inspected = None;
            }
            Ok(Err(error)) => {
                self.paso3_status = format!("paso 3 failed: {error}");
            }
            Err(TryRecvError::Empty) => {
                self.paso3_receiver = Some(receiver);
                ctx.request_repaint_after(Duration::from_millis(50));
            }
            Err(TryRecvError::Disconnected) => {
                self.paso3_status = "paso 3 worker stopped unexpectedly.".to_string();
            }
        }
    }

    fn run_selected(&mut self) {
        self.run_jobs(vec![self.selected]);
    }

    fn run_all(&mut self) {
        self.run_jobs((0..self.entry_count()).collect());
    }

    /// Lanza el paso 1 (nuevo algoritmo, sin existentes fijas) para las entradas dadas.
    fn run_jobs(&mut self, indices: Vec<usize>) {
        if indices.is_empty() || self.receiver.is_some() {
            return;
        }
        let Some(datos) = self.datos.clone() else {
            self.status =
                "data/ not found: run `reto datos` first (the UI now needs the CONAFOR data)."
                    .to_string();
            return;
        };
        let Some(config) = self.reto_config.clone() else {
            self.status =
                "config/default.json not found: the UI now needs the model configuration."
                    .to_string();
            return;
        };

        let jobs: Vec<(usize, String, f64)> = indices
            .into_iter()
            .filter_map(|index| {
                self.entrada_instancia(index)
                    .map(|(nombre, area)| (index, nombre, area))
            })
            .collect();
        if jobs.is_empty() {
            self.status =
                "data/poligonos.csv does not match the polygons of bins/polygons.bin.".to_string();
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.receiver = Some(receiver);
        self.pending = jobs.len();
        self.status = format!("Solving {} instance(s) (paso 1)...", jobs.len());

        thread::spawn(move || {
            for (index, nombre, area) in jobs {
                let resultado = correr_paso1(nombre, area, &datos, &config);
                if sender.send((index, resultado)).is_err() {
                    break;
                }
            }
        });
    }

    fn poll_worker(&mut self, ctx: &egui::Context) {
        let Some(receiver) = self.receiver.take() else {
            return;
        };

        let mut disconnected = false;
        loop {
            match receiver.try_recv() {
                Ok((index, Ok(view))) => {
                    self.status = format!(
                        "{}: Z {:.2} -> {:.2}, $/ha {:.0}",
                        view.nombre, view.z_inicial, view.z, view.costo_ha
                    );
                    self.resultados[index] = Some(view);
                    self.pending = self.pending.saturating_sub(1);
                }
                Ok((index, Err(error))) => {
                    self.status = format!("paso 1 failed ({index}): {error}");
                    self.pending = self.pending.saturating_sub(1);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }

        if disconnected || self.pending == 0 {
            self.receiver = None;
        } else {
            self.receiver = Some(receiver);
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Reforestation");
        let (epsilon, presupuesto) = self
            .reto_config
            .as_ref()
            .map_or((0.10, 12_500.0), |c| (c.epsilon, c.presupuesto_ha));
        ui.label(format!(
            "Count tolerance +/- {:.0}%   |   budget {:.0} $/ha",
            epsilon * 100.0,
            presupuesto
        ));
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Run selected").clicked() {
                self.run_selected();
            }
            if ui.button("Run all").clicked() {
                self.run_all();
            }
        });
        if self.pending > 0 {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(format!("{:} left", self.pending));
            });
        }
        ui.label(&self.status);
        ui.separator();

        let entries: Vec<String> = (0..self.entry_count())
            .map(|index| {
                let done = if self.resultados[index].is_some() {
                    " [ok]"
                } else {
                    ""
                };
                if self.is_custom(index) {
                    format!(
                        "Custom ({}x{}){}",
                        self.custom_width, self.custom_height, done
                    )
                } else {
                    let polygon = &self.polygons[index];
                    format!(
                        "{} ({:.2} ha){}",
                        polygon.get_polygon_id(),
                        polygon.get_area_hectare(),
                        done
                    )
                }
            })
            .collect();

        ui.label("Polygons");
        egui::ScrollArea::vertical()
            .id_salt("polygon_list")
            .max_height(200.0)
            .show(ui, |ui| {
                for (index, text) in entries.iter().enumerate() {
                    if ui.selectable_label(self.selected == index, text).clicked() {
                        self.selected = index;
                        self.hovered = None;
                        self.inspected = None;
                    }
                }
            });

        if self.is_custom(self.selected) {
            ui.separator();
            ui.label("Custom polygon dimensions");
            let mut changed = false;
            ui.horizontal(|ui| {
                ui.label("width");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.custom_width).range(1..=300))
                    .changed();
                ui.label("height");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.custom_height).range(1..=300))
                    .changed();
            });
            let cells = self.custom_width * self.custom_height;
            let density = self.custom_density();
            let area = if density > 0.0 {
                cells as f64 / density
            } else {
                0.0
            };
            ui.weak(format!(
                "{} cells | {:.2} ha | {:.0} plants/ha",
                cells, area, density
            ));
            if changed {
                self.resultados[self.selected] = None;
            }
        }

        ui.separator();
        ui.label(egui::RichText::new("Paso 3 — ajuste al polígono real").strong());
        ui.horizontal(|ui| {
            ui.label("escenarios");
            ui.add(egui::DragValue::new(&mut self.n_omega).range(1..=1000));
        });
        let datos_listo = self.datos.is_some() && self.reto_config.is_some();
        if !datos_listo {
            ui.weak("needs data/ and config/default.json");
        } else if ui.button("Run paso 3").clicked() {
            self.run_paso3();
        }
        if self.paso3_receiver.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.weak("resolviendo escenarios...");
            });
        }
        ui.label(&self.paso3_status);

        ui.separator();
        ui.label("Species filter");
        egui::ScrollArea::vertical()
            .id_salt("species_list")
            .max_height(220.0)
            .show(ui, |ui| {
                let counts = self.resultados[self.selected].as_ref().map(|view| view.d);
                for (index, plant) in self.plants.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, species_color(index));
                        ui.checkbox(&mut self.visible[index], plant.get_specie());
                        if let Some(counts) = &counts {
                            ui.weak(format!("{}", counts[index]));
                        }
                    });
                }
            });

        ui.separator();
        self.inspector(ui);
    }

    fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.label("Cell inspector");
        let cell = self.inspected.or(self.hovered);
        let Some(cell) = cell else {
            ui.label("Hover or click a cell.");
            return;
        };
        if self.vista == Vista::Paso3 {
            let Some(view) = &self.paso3 else {
                ui.label("No paso 3 result.");
                return;
            };
            self.inspector_celda(
                ui,
                cell,
                &view.posiciones,
                &view.phi,
                Some(&view.existentes),
            );
            return;
        }
        let Some(view) = &self.resultados[self.selected] else {
            ui.label("No result for this polygon.");
            return;
        };
        let phi = if self.vista == Vista::Antes {
            &view.phi_inicial
        } else {
            &view.phi
        };
        self.inspector_celda(ui, cell, &view.posiciones, phi, None);
    }

    /// Detalle de la celda `cell` de una malla en metros.
    fn inspector_celda(
        &self,
        ui: &mut egui::Ui,
        cell: usize,
        posiciones: &[Posicion],
        phi: &[u8],
        existentes: Option<&[bool]>,
    ) {
        if cell >= posiciones.len() {
            return;
        }
        let pos = &posiciones[cell];
        ui.label(format!("pos {}  fila {}  col {}", cell, pos.fila, pos.col));
        ui.label(format!("({:.2} m, {:.2} m)", pos.x, pos.y));
        if let Some(existentes) = existentes {
            if existentes[cell] {
                ui.colored_label(Color32::from_rgb(255, 200, 80), "existing plant (fixed)");
            } else {
                ui.weak("free position");
            }
        }
        match phi[cell] {
            0 => {
                ui.label("empty");
            }
            code => {
                let species = code as usize - 1;
                let plant = &self.plants[species];
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, species_color(species));
                    ui.label(plant.get_specie());
                });
                ui.label(format!("Unitary cost: {:.2}", plant.get_unitary_cost()));
                ui.label(format!(
                    "Root spread radius: {}",
                    plant.get_root_spread_radius()
                ));
                ui.label(format!(
                    "Interaction index: {:.2}",
                    plant.get_interaction_index()
                ));
            }
        }
    }

    fn center(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("View:").strong());
            ui.selectable_value(&mut self.vista, Vista::Antes, "Before");
            ui.selectable_value(&mut self.vista, Vista::Despues, "After");
            ui.selectable_value(&mut self.vista, Vista::Paso3, "Paso 3");
            match self.vista {
                Vista::Antes => ui.weak("serpentina (inicio del tabú)"),
                Vista::Despues => ui.weak("paso 1: γ·Z + Σθ·D en la banda +10%"),
                Vista::Paso3 => ui.weak("existentes fijas + cuotas de compra"),
            }
        });
        ui.separator();
        match self.vista {
            Vista::Paso3 => self.center_paso3(ui),
            _ => self.center_banda(ui),
        }
    }

    fn center_banda(&mut self, ui: &mut egui::Ui) {
        let Some(view) = self.resultados[self.selected].clone() else {
            ui.centered_and_justified(|ui| {
                ui.label("No result yet. Click \"Run selected\" or \"Run all\".");
            });
            return;
        };

        self.stats(ui, &view);
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("counts")
            .max_height(210.0)
            .show(ui, |ui| {
                self.counts_table(ui, &view);
            });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Grid:").strong());
            ui.weak("colored = planted   ·   gray = empty   ·   drawn in meters");
        });
        let phi = if self.vista == Vista::Antes {
            view.phi_inicial.clone()
        } else {
            view.phi.clone()
        };
        self.draw_malla_cells(ui, &view.posiciones, view.separacion, &phi, None);
    }

    fn center_paso3(&mut self, ui: &mut egui::Ui) {
        let Some(view) = self.paso3.clone() else {
            ui.centered_and_justified(|ui| {
                ui.label("No paso 3 result. Pick a polygon and click \"Run paso 3\".");
            });
            return;
        };
        if view.indice != self.selected {
            ui.centered_and_justified(|ui| {
                ui.label(format!(
                    "This paso 3 result is for {}. Select it (or run paso 3 again for this polygon).",
                    view.nombre
                ));
            });
            return;
        }

        ui.horizontal_wrapped(|ui| {
            ui.strong(&view.nombre);
            ui.label(format!("{:.2} ha", view.area));
            ui.separator();
            ui.label(format!("caso: {}", view.fila.caso.como_str()));
            ui.label(format!(
                "T {} | V_lib {} | M {}",
                view.fila.t, view.fila.v_lib, view.fila.m
            ));
            ui.separator();
            ui.label(format!(
                "Z {:.2} | Z/planta {:.4}",
                view.fila.z, view.fila.z_por_planta
            ));
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "Z sobre {} escenarios: media {:.2}  sd {:.2}  p50 {:.2}  p95 {:.2}",
                view.n_omega, view.z_media, view.z_sd, view.z_p50, view.z_p95
            ));
            ui.separator();
            ui.label(format!(
                "casos: {}/{} (usual/corto/sobra)",
                view.casos[0],
                view.casos[1] + view.casos[2]
            ));
        });
        ui.horizontal_wrapped(|ui| {
            let dentro = view.costo_ha <= view.presupuesto_ha + 1e-9;
            let color = if dentro {
                Color32::from_rgb(80, 180, 80)
            } else {
                Color32::from_rgb(220, 90, 90)
            };
            ui.colored_label(
                color,
                format!(
                    "compra ${:.0} (${:.0}/ha, budget ${:.0}/ha) {}",
                    view.costo_compra,
                    view.costo_ha,
                    view.presupuesto_ha,
                    if dentro { "ok" } else { "OVER" }
                ),
            );
            ui.separator();
            ui.weak(format!(
                "base: {} | q: {} | iter {}",
                view.fuente_base, view.fuente_q, view.fila.iteraciones
            ));
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("paso3_counts")
            .max_height(230.0)
            .show(ui, |ui| {
                self.paso3_counts_table(ui, &view);
            });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Grid:").strong());
            ui.weak("colored = new plants   ·   ringed = existing (fixed)   ·   gray = empty");
        });
        self.draw_malla_cells(
            ui,
            &view.posiciones,
            view.separacion,
            &view.phi,
            Some(&view.existentes),
        );
    }

    fn paso3_counts_table(&self, ui: &mut egui::Ui, view: &Paso3View) {
        let mut nuevas = [0usize; N_ESPECIES];
        for &code in view.phi.iter() {
            if code > 0 {
                nuevas[code as usize - 1] += 1;
            }
        }
        egui::Grid::new("paso3_grid")
            .striped(true)
            .num_columns(7)
            .show(ui, |ui| {
                ui.strong("species");
                ui.strong("X");
                ui.strong("D* [L..U]");
                ui.strong("q");
                ui.strong("n");
                ui.strong("new");
                ui.strong("total");
                ui.end_row();

                for (index, plant) in self.plants.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(11.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, species_color(index));
                        ui.label(plant.get_specie());
                    });
                    ui.label(format!("{}", view.x[index]));
                    ui.label(format!(
                        "{} [{}..{}]",
                        view.d[index], view.banda.l[index], view.banda.u[index]
                    ));
                    ui.label(format!("{}", view.q[index]));
                    ui.label(format!("{}", view.n[index]));
                    ui.label(format!("{}", nuevas[index]));
                    ui.label(format!("{}", view.x[index] as usize + nuevas[index]));
                    ui.end_row();
                }

                ui.strong("total");
                for valor in [
                    view.x.iter().map(|&x| x as usize).sum::<usize>(),
                    view.d.iter().map(|&x| x as usize).sum::<usize>(),
                    view.q.iter().map(|&x| x as usize).sum::<usize>(),
                    view.n.iter().sum::<usize>(),
                    nuevas.iter().sum::<usize>(),
                    view.posiciones.len(),
                ] {
                    ui.strong(format!("{valor}"));
                }
                ui.end_row();
            });
    }

    fn counts_table(&self, ui: &mut egui::Ui, view: &Paso1View) {
        egui::Grid::new("counts_grid")
            .striped(true)
            .num_columns(5)
            .show(ui, |ui| {
                ui.strong("species");
                ui.strong("before");
                ui.strong("after");
                ui.strong("D̄");
                ui.strong("band [L..U]");
                ui.end_row();

                for (index, plant) in self.plants.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(11.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, species_color(index));
                        ui.label(plant.get_specie());
                    });
                    ui.label(format!("{}", view.d_inicial[index]));
                    ui.label(format!("{}", view.d[index]));
                    ui.label(format!("{}", view.nominal[index]));
                    ui.label(format!("{}..{}", view.banda.l[index], view.banda.u[index]));
                    ui.end_row();
                }

                let before: usize = view.d_inicial.iter().map(|&x| x as usize).sum();
                let after: usize = view.d.iter().map(|&x| x as usize).sum();
                ui.strong("total");
                ui.strong(format!("{before}"));
                ui.strong(format!("{after}"));
                ui.label(format!("{}", view.nominal.iter().sum::<usize>()));
                ui.label(format!(
                    "{}..{}",
                    view.banda.l.iter().sum::<usize>(),
                    view.banda.u.iter().sum::<usize>()
                ));
                ui.end_row();
            });
    }

    fn stats(&self, ui: &mut egui::Ui, view: &Paso1View) {
        let show_before = self.vista == Vista::Antes;
        let d = if show_before {
            &view.d_inicial
        } else {
            &view.d
        };
        let plantados: usize = d.iter().map(|&x| x as usize).sum();
        let (z, mortalidad, costo) = if show_before {
            (view.z_inicial, view.mortalidad_inicial, view.costo_inicial)
        } else {
            (view.z, view.mortalidad, view.costo)
        };

        ui.horizontal_wrapped(|ui| {
            ui.strong(&view.nombre);
            ui.label(format!("{:.2} ha", view.area));
            ui.separator();
            ui.label(format!("grid {}x{}", view.columnas, view.filas));
            ui.label(format!(
                "placed {plantados} (D_tot {}, band [{}..{}])",
                view.posiciones.len(),
                view.banda.l.iter().sum::<usize>(),
                view.banda.u.iter().sum::<usize>(),
            ));
        });
        ui.horizontal_wrapped(|ui| {
            if show_before {
                ui.label(format!("Z {:.2} | Σθ·D {:.2}", z, mortalidad));
            } else {
                ui.label(format!(
                    "Z {:.2} -> {:.2} | Z/planta {:.4}",
                    view.z_inicial, view.z, view.z_por_planta
                ));
                ui.separator();
                ui.label(format!("Σθ·D {:.2} | γ {:.3}", mortalidad, view.gamma));
            }
            ui.separator();
            let dentro = view.costo_ha <= view.presupuesto_ha + 1e-9;
            let budget_color = if dentro {
                Color32::from_rgb(80, 180, 80)
            } else {
                Color32::from_rgb(220, 90, 90)
            };
            ui.colored_label(
                budget_color,
                format!(
                    "$/ha {:.0} (budget {:.0}) {}",
                    if show_before {
                        view.costo_inicial / view.area
                    } else {
                        view.costo_ha
                    },
                    view.presupuesto_ha,
                    if dentro { "ok" } else { "OVER" }
                ),
            );
            ui.separator();
            ui.label(format!("Σc·D {:.0}", costo));
        });
        if !show_before {
            ui.horizontal_wrapped(|ui| {
                ui.weak(format!(
                    "iters {} | {:.2}s | objetivo paso 1 {:.2}",
                    view.iteraciones, view.tiempo_s, view.objetivo
                ));
            });
        }
    }

    /// Dibuja la malla en metros: φ con la codificación 0/vacía, especie j ↔ j + 1. Si se pasa
    /// `existentes`, esas posiciones se marcan con anillo + punto (fijas).
    fn draw_malla_cells(
        &mut self,
        ui: &mut egui::Ui,
        posiciones: &[Posicion],
        separacion: f64,
        phi: &[u8],
        existentes: Option<&[bool]>,
    ) {
        let available = ui.available_size();
        let (response, painter) = ui.allocate_painter(available, Sense::click());
        painter.rect_filled(response.rect, 4.0, Color32::from_gray(18));

        if posiciones.is_empty() {
            return;
        }

        let sqrt3 = 3f32.sqrt();
        let d = separacion.max(0.01) as f32;
        let puntos: Vec<Vec2> = posiciones
            .iter()
            .map(|p| Vec2::new(p.x as f32, -(p.y as f32)))
            .collect();
        let mut min = puntos[0];
        let mut max = puntos[0];
        for &p in &puntos {
            min = min.min(p);
            max = max.max(p);
        }
        let pad = 8.0;
        let avail_w = (response.rect.width() - 2.0 * pad).max(1.0);
        let avail_h = (response.rect.height() - 2.0 * pad).max(1.0);
        let escala = (avail_w / (max.x - min.x + 2.0 * d))
            .min(avail_h / (max.y - min.y + 2.0 * d))
            .max(0.01);
        let radio = ((d / sqrt3) * escala).clamp(1.0, 48.0);
        let centro_dibujo = (min + max) * 0.5 * escala;
        let origen: Vec2 = response.rect.center().to_vec2() - centro_dibujo;

        let hover_pos = response.hover_pos();
        let mut hovered_cell = None;
        let mut hovered_dist = f32::INFINITY;

        for (cell, &punto) in puntos.iter().enumerate() {
            let center = (punto * escala + origen).to_pos2();

            if let Some(pos) = hover_pos {
                let dist = pos.distance(center);
                if dist <= radio && dist < hovered_dist {
                    hovered_dist = dist;
                    hovered_cell = Some(cell);
                }
            }

            let points = hex_points(center, radio);
            let color = match phi[cell] {
                0 => Color32::from_gray(60),
                code => {
                    let base = species_color(code as usize - 1);
                    if self.visible[code as usize - 1] {
                        base
                    } else {
                        base.gamma_multiply(0.15)
                    }
                }
            };
            painter.add(Shape::convex_polygon(
                points.clone(),
                color,
                Stroke::new(1.0, Color32::from_gray(30)),
            ));

            if let Some(existentes) = existentes {
                if existentes[cell] {
                    painter.add(Shape::closed_line(points, Stroke::new(1.5, Color32::WHITE)));
                    painter.circle_filled(
                        center,
                        (radio * 0.22).max(1.0),
                        Color32::from_rgb(255, 240, 200),
                    );
                }
            }

            if hovered_cell == Some(cell) || self.inspected == Some(cell) {
                painter.add(Shape::closed_line(
                    hex_points(center, radio),
                    Stroke::new(2.0, Color32::WHITE),
                ));
            }
        }

        self.hovered = hovered_cell;
        if response.clicked() {
            self.inspected = hovered_cell;
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_worker(&ctx);
        self.poll_paso3(&ctx);

        egui::Panel::left("controls")
            .resizable(true)
            .default_size(320.0)
            .show(ui, |ui| {
                self.controls(ui);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            self.center(ui);
        });
    }
}

fn hex_points(center: Pos2, size: f32) -> Vec<Pos2> {
    (0..6)
        .map(|i| {
            let angle = std::f32::consts::PI / 180.0 * (60.0 * i as f32 + 30.0);
            center + Vec2::new(size * angle.cos(), size * angle.sin())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_d_paso1_del_csv_publicado() {
        let d = leer_d_paso1(Path::new("results/paso1_composicion.csv"), "map_27")
            .expect("map_27 tiene fila publicada");
        assert_eq!(d.iter().map(|&x| x as usize).sum::<usize>(), 842);
        assert_eq!(d[1], 226);
    }

    #[test]
    fn lee_d_paso1_falla_para_instancia_desconocida() {
        let error = leer_d_paso1(Path::new("results/paso1_composicion.csv"), "no_existe")
            .expect_err("sin fila");
        assert!(error.contains("no_existe"), "{error}");
    }

    #[test]
    fn paso1_ui_cumple_banda_y_serpentina() {
        let datos = Datos::cargar(Path::new("data")).expect("data/ cargado");
        let mut config = Config::cargar(Path::new("config/default.json")).expect("config");
        config.tabu.max_iter = 400;
        config.tabu.max_sin_mejora = 80;

        let view =
            correr_paso1("1ha".to_string(), 1.0, &datos, &config).expect("paso 1 corre para 1ha");

        assert_eq!(view.phi.len(), view.posiciones.len());
        assert_eq!(view.phi_inicial.len(), view.posiciones.len());

        // Ambos patrones tienen D_tot plantas y cumplen la banda.
        for j in 0..N_ESPECIES {
            for d in [&view.d, &view.d_inicial] {
                assert!(
                    (view.banda.l[j]..=view.banda.u[j]).contains(&(d[j] as usize)),
                    "especie {j}: D_j = {} fuera de [{}..{}]",
                    d[j],
                    view.banda.l[j],
                    view.banda.u[j]
                );
            }
        }
        let total: usize = view.d.iter().map(|&x| x as usize).sum();
        assert_eq!(total, view.posiciones.len());

        assert!(view.z > 0.0 && view.z_inicial > 0.0);
        assert!(view.z_por_planta > 0.0);
        assert!(view.costo_ha > 0.0 && view.costo_inicial > 0.0);
        assert!(view.iteraciones > 0);
    }

    #[test]
    fn paso3_respeta_cuotas_fijas_y_m() {
        let datos = Datos::cargar(Path::new("data")).expect("data/ cargado");
        let mut config = Config::cargar(Path::new("config/default.json")).expect("config");
        config.tabu.max_iter = 400;
        config.tabu.max_sin_mejora = 80;

        let view = correr_paso3(0, "1ha".to_string(), 1.0, &datos, &config, 2, None)
            .expect("paso 3 corre para 1ha");

        assert_eq!(view.posiciones.len(), view.phi.len());
        assert_eq!(view.existentes.iter().filter(|&&f| f).count(), view.fila.t);
        assert_eq!(view.n_omega, 2);

        let (n, m, caso) = paso3::cuotas(&view.q, &view.d, &view.x, view.fila.v_lib);
        assert_eq!(view.n, n);
        assert_eq!(view.fila.m, m);
        assert_eq!(view.fila.caso, caso);

        // Las existentes nunca se mueven y las nuevas respetan n_j.
        let existentes: std::collections::HashSet<usize> = (0..view.phi.len())
            .filter(|&u| view.existentes[u])
            .collect();
        assert_eq!(existentes.len(), view.fila.t);
        let mut nuevas = [0usize; N_ESPECIES];
        for (u, &code) in view.phi.iter().enumerate() {
            if code > 0 && !existentes.contains(&u) {
                nuevas[code as usize - 1] += 1;
            }
        }
        assert_eq!(nuevas.to_vec(), n);

        let plantados: usize = view.phi.iter().filter(|&&c| c > 0).count();
        assert_eq!(plantados, view.fila.t + m);
        assert!(view.z_media > 0.0);
        assert!(view.z_p95 >= view.z_p50);
    }
}
