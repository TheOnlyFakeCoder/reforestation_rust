use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use eframe::egui;
use egui::{Color32, Pos2, Sense, Shape, Stroke, Vec2};

use reforestation::tabu::{Allocation, TabuConfig, TabuSearch, COUNT_TOLERANCE};
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
    allocations: Vec<Option<Allocation>>,
    selected: usize,
    config: TabuConfig,
    visible: Vec<bool>,
    view_before: bool,
    hovered: Option<usize>,
    inspected: Option<usize>,
    receiver: Option<Receiver<(usize, Allocation)>>,
    pending: usize,
    status: String,
}

impl App {
    fn new() -> Self {
        let plants = read_plants_file("bins/plants.bin");
        let polygons = read_polygons_file("bins/polygons.bin");
        let visible = vec![true; plants.len()];
        let allocations = vec![None; polygons.len()];

        let mut app = App {
            plants,
            polygons,
            allocations,
            selected: 0,
            config: TabuConfig::default(),
            visible,
            view_before: false,
            hovered: None,
            inspected: None,
            receiver: None,
            pending: 0,
            status: "Press \"Run selected\" to optimize the highlighted polygon.".to_string(),
        };
        app.run_selected();
        app
    }

    fn run_selected(&mut self) {
        if self.polygons.is_empty() {
            return;
        }
        let polygon = &self.polygons[self.selected];
        let allocation = TabuSearch::new(&self.plants, polygon, self.config)
            .with_row_sequence(ROW_SEQUENCE.to_vec())
            .solve();
        self.status = format!(
            "{} optimized: eco {:.2}, $/ha {:.0}",
            polygon.get_polygon_id(),
            allocation.best_cost,
            allocation.monetary_cost_per_hectare
        );
        self.allocations[self.selected] = Some(allocation);
    }

    fn run_all(&mut self) {
        if self.polygons.is_empty() || self.receiver.is_some() {
            return;
        }
        let plants = self.plants.clone();
        let polygons = self.polygons.clone();
        let config = self.config;
        let (sender, receiver) = mpsc::channel();

        self.receiver = Some(receiver);
        self.pending = polygons.len();
        self.allocations = vec![None; polygons.len()];
        self.status = "Solving all polygons...".to_string();

        thread::spawn(move || {
            for (index, polygon) in polygons.iter().enumerate() {
                let allocation = TabuSearch::new(&plants, polygon, config)
                    .with_row_sequence(ROW_SEQUENCE.to_vec())
                    .solve();
                if sender.send((index, allocation)).is_err() {
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
                Ok((index, allocation)) => {
                    self.allocations[index] = Some(allocation);
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
            self.status = "All polygons optimized.".to_string();
        } else {
            self.receiver = Some(receiver);
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Reforestation");
        ui.label(format!(
            "Count tolerance +/- {:.0}%   |   budget {:.0} $/ha",
            COUNT_TOLERANCE * 100.0,
            self.config.budget_per_hectare
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

        ui.label("Polygons");
        egui::ScrollArea::vertical()
            .id_salt("polygon_list")
            .max_height(200.0)
            .show(ui, |ui| {
                for (index, polygon) in self.polygons.iter().enumerate() {
                    let done = if self.allocations[index].is_some() {
                        " [ok]"
                    } else {
                        ""
                    };
                    let text = format!(
                        "{} ({:.2} ha){}",
                        polygon.get_polygon_id(),
                        polygon.get_area_hectare(),
                        done
                    );
                    if ui.selectable_label(self.selected == index, text).clicked() {
                        self.selected = index;
                        self.hovered = None;
                        self.inspected = None;
                    }
                }
            });

        ui.separator();
        ui.label("Species filter");
        egui::ScrollArea::vertical()
            .id_salt("species_list")
            .max_height(220.0)
            .show(ui, |ui| {
                let counts = self.allocations[self.selected]
                    .as_ref()
                    .map(|allocation| allocation.counts.clone());
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
        let Some(allocation) = &self.allocations[self.selected] else {
            ui.label("No allocation for this polygon.");
            return;
        };
        if cell >= allocation.coords.len() {
            return;
        }
        let coord = allocation.coords[cell];
        let (col, row) = coord.to_offset();
        ui.label(format!(
            "axial ({}, {})  offset ({}, {})",
            coord.q, coord.r, col, row
        ));

        match allocation.species[cell] {
            Some(species) => {
                let plant = &self.plants[species];
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, species_color(species));
                    ui.label(plant.get_specie());
                });
                ui.label(format!("Unitary cost: {:.2}", plant.get_unitary_cost()));
                ui.label(format!("Per hectare: {}", plant.get_per_hectare()));
                ui.label(format!("Height: {:?}", plant.get_height()));
                ui.label(format!("Root depth: {:?}", plant.get_root_depth()));
                ui.label(format!(
                    "Root spread radius: {}",
                    plant.get_root_spread_radius()
                ));
                ui.label(format!(
                    "Interaction index: {:.2}",
                    plant.get_interaction_index()
                ));
            }
            None => {
                ui.label("empty cell");
            }
        }
    }

    fn center(&mut self, ui: &mut egui::Ui) {
        let Some(allocation) = self.allocations[self.selected].clone() else {
            ui.centered_and_justified(|ui| {
                ui.label("No allocation yet. Click \"Run selected\" or \"Run all\".");
            });
            return;
        };

        self.stats(ui, &allocation);
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("counts")
            .max_height(210.0)
            .show(ui, |ui| {
                self.counts_table(ui, &allocation);
            });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Grid view:").strong());
            ui.selectable_value(&mut self.view_before, true, "Before");
            ui.selectable_value(&mut self.view_before, false, "After");
            if self.view_before {
                ui.weak("initial placement");
            } else {
                ui.weak("Tabu optimized");
            }
        });
        let layout = if self.view_before {
            &allocation.initial_species
        } else {
            &allocation.species
        };
        self.draw_cells(ui, &allocation, layout);
    }

    fn counts_table(&self, ui: &mut egui::Ui, allocation: &Allocation) {
        egui::Grid::new("counts_grid")
            .striped(true)
            .num_columns(5)
            .show(ui, |ui| {
                ui.strong("species");
                ui.strong("before");
                ui.strong("after");
                ui.strong("target");
                ui.strong("band 90-110%");
                ui.end_row();

                for (index, plant) in self.plants.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(11.0), Sense::hover());
                        ui.painter().rect_filled(rect, 2.0, species_color(index));
                        ui.label(plant.get_specie());
                    });
                    ui.label(format!("{}", allocation.initial_counts[index]));
                    ui.label(format!("{}", allocation.counts[index]));
                    ui.label(format!("{}", allocation.target[index]));
                    ui.label(format!(
                        "{}..{}",
                        allocation.min_counts[index], allocation.max_counts[index]
                    ));
                    ui.end_row();
                }

                let before: usize = allocation.initial_counts.iter().sum();
                let after: usize = allocation.counts.iter().sum();
                ui.strong("total");
                ui.strong(format!("{before}"));
                ui.strong(format!("{after}"));
                ui.label(format!("{}", allocation.target.iter().sum::<usize>()));
                ui.label(format!(
                    "{}..{}",
                    allocation.min_counts.iter().sum::<usize>(),
                    allocation.max_counts.iter().sum::<usize>()
                ));
                ui.end_row();
            });
    }

    fn stats(&self, ui: &mut egui::Ui, allocation: &Allocation) {
        let polygon = &self.polygons[self.selected];
        let show_before = self.view_before;

        let initial_total: f64 = self
            .plants
            .iter()
            .zip(&allocation.initial_counts)
            .map(|(plant, &count)| plant.get_unitary_cost() * count as f64)
            .sum();
        let initial_per_hectare = if polygon.get_area_hectare() > 0.0 {
            initial_total / polygon.get_area_hectare()
        } else {
            0.0
        };

        let placed = if show_before {
            allocation.initial_counts.iter().sum::<usize>()
        } else {
            allocation.placed()
        };
        let (cost, per_hectare) = if show_before {
            (allocation.initial_cost, initial_per_hectare)
        } else {
            (allocation.best_cost, allocation.monetary_cost_per_hectare)
        };
        let within_budget = per_hectare <= allocation.budget_per_hectare + 1e-9;

        ui.horizontal_wrapped(|ui| {
            ui.strong(polygon.get_polygon_id());
            ui.label(format!("{:.2} ha", polygon.get_area_hectare()));
            ui.separator();
            ui.label(format!(
                "grid {}x{}",
                allocation.dimension, allocation.dimension
            ));
            ui.label(format!(
                "placed {} (target {}, band [{}..{}])",
                placed,
                allocation.target.iter().sum::<usize>(),
                allocation.min_counts.iter().sum::<usize>(),
                allocation.max_counts.iter().sum::<usize>(),
            ));
        });
        ui.horizontal_wrapped(|ui| {
            if show_before {
                ui.label(format!("eco {:.2}", cost));
            } else {
                ui.label(format!("eco {:.2} -> {:.2}", allocation.initial_cost, cost));
            }
            ui.separator();
            let budget_color = if within_budget {
                Color32::from_rgb(80, 180, 80)
            } else {
                Color32::from_rgb(220, 90, 90)
            };
            ui.colored_label(
                budget_color,
                format!(
                    "${:.0}/ha (budget ${:.0}) {}",
                    per_hectare,
                    allocation.budget_per_hectare,
                    if within_budget { "ok" } else { "OVER" }
                ),
            );
            if !show_before {
                ui.separator();
                ui.label(format!(
                    "iters {} | improved {} | tabu-hits {}",
                    allocation.iterations, allocation.improvements, allocation.tabu_hits
                ));
            }
        });
    }

    fn draw_cells(
        &mut self,
        ui: &mut egui::Ui,
        allocation: &Allocation,
        species_layout: &[Option<usize>],
    ) {
        let available = ui.available_size();
        let (response, painter) = ui.allocate_painter(available, Sense::click());
        painter.rect_filled(response.rect, 4.0, Color32::from_gray(18));

        if allocation.coords.is_empty() {
            return;
        }

        let sqrt3 = 3f32.sqrt();
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for coord in &allocation.coords {
            let x = sqrt3 * (coord.q as f32 + coord.r as f32 / 2.0);
            let y = 1.5 * coord.r as f32;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }

        let width_units = max_x - min_x;
        let height_units = max_y - min_y;
        let pad = 8.0;
        let avail_w = (response.rect.width() - 2.0 * pad).max(1.0);
        let avail_h = (response.rect.height() - 2.0 * pad).max(1.0);
        let size_x = avail_w / (width_units + sqrt3);
        let size_y = avail_h / (height_units + 2.0);
        let size = size_x.min(size_y).clamp(1.0, 48.0);

        let drawn_w = (width_units + sqrt3) * size;
        let drawn_h = (height_units + 2.0) * size;
        let origin = response.rect.center()
            - Vec2::new(drawn_w / 2.0, drawn_h / 2.0)
            - Vec2::new(min_x * size - sqrt3 * size / 2.0, min_y * size - size);

        let hover_pos = response.hover_pos();
        let mut hovered_cell = None;
        let mut hovered_dist = f32::INFINITY;

        for (cell, coord) in allocation.coords.iter().enumerate() {
            let center = origin
                + Vec2::new(
                    sqrt3 * (coord.q as f32 + coord.r as f32 / 2.0) * size,
                    1.5 * coord.r as f32 * size,
                );

            if let Some(pos) = hover_pos {
                let dist = pos.distance(center);
                if dist <= size && dist < hovered_dist {
                    hovered_dist = dist;
                    hovered_cell = Some(cell);
                }
            }

            let points = hex_points(center, size);

            match species_layout[cell] {
                Some(species) => {
                    let base = species_color(species);
                    let color = if self.visible[species] {
                        base
                    } else {
                        base.gamma_multiply(0.15)
                    };
                    painter.add(Shape::convex_polygon(
                        points.clone(),
                        color,
                        Stroke::new(1.0, Color32::from_gray(30)),
                    ));
                }
                None => {
                    painter.add(Shape::convex_polygon(
                        points.clone(),
                        Color32::from_gray(60),
                        Stroke::new(1.0, Color32::from_gray(30)),
                    ));
                }
            }

            if hovered_cell == Some(cell) || self.inspected == Some(cell) {
                painter.add(Shape::closed_line(points, Stroke::new(2.0, Color32::WHITE)));
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
