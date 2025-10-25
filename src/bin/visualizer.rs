// Multi-Vehicle Navigation Visualizer with egui
// Runs simulation automatically and displays results
// Run with: cargo run --bin visualizer

use examen_parcial::map::Map;
use examen_parcial::simulation::{Simulation, MultiVehicleSimulationResult, VehicleResult};
use examen_parcial::vehicle::VehicleType;
use macroquad::prelude::*;
use std::fs;
use std::io::Write;

const WINDOW_WIDTH: f32 = 1800.0;
const WINDOW_HEIGHT: f32 = 1000.0;
const SIDEBAR_WIDTH: f32 = 450.0;
const MAP_PADDING: f32 = 40.0;

/// Application state
enum AppState {
    Configuration,
    RunningSimulation,
    Visualization,
}

/// Configuration for a single vehicle before simulation
#[derive(Clone)]
struct VehicleConfig {
    vehicle_type: VehicleType,
    position_x: f32,
    position_y: f32,
    angle_degrees: f32,
    velocity_percentage: f32, // 0.0 to 1.0
    use_random: bool,
}

impl VehicleConfig {
    fn new_random(vehicle_type: VehicleType, map: &Map) -> Self {
        Self {
            vehicle_type,
            position_x: map.random_start_position().x as f32,
            position_y: map.random_start_position().y as f32,
            angle_degrees: map.random_start_angle().to_degrees() as f32,
            velocity_percentage: (map.random_start_velocity_percentage() * 100.0) as f32,
            use_random: true,
        }
    }

    fn randomize(&mut self, map: &Map) {
        let pos = map.random_start_position();
        self.position_x = pos.x as f32;
        self.position_y = pos.y as f32;
        self.angle_degrees = map.random_start_angle().to_degrees() as f32;
        self.velocity_percentage = (map.random_start_velocity_percentage() * 100.0) as f32;
        self.use_random = true;
    }
}

/// Run the multi-vehicle simulation and save results
fn run_simulation(configs: &[VehicleConfig]) -> MultiVehicleSimulationResult {
    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║   EJECUTANDO SIMULACIÓN DE NAVEGACIÓN DIFUSA         ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Create map (1000x800, target at top center: 500,700)
    let map = Map::new(1000.0, 800.0, 500.0, 700.0);

    let dt = 0.05; // 50ms time step
    let max_time = 600.0;

    // Create simulations from configs
    let mut simulations: Vec<Simulation> = configs.iter()
        .map(|config| {
            use examen_parcial::vehicle::create_vehicle_preset;
            use examen_parcial::navigation::NavigationController;
            use examen_parcial::map::Point;
            use examen_parcial::vehicle::Vehicle;

            let characteristics = create_vehicle_preset(config.vehicle_type);
            let initial_pos = Point::new(config.position_x as f64, config.position_y as f64);
            let initial_angle = config.angle_degrees.to_radians() as f64;

            let mut vehicle = Vehicle::new(
                config.vehicle_type,
                characteristics.clone(),
                initial_pos,
                initial_angle,
            );

            // Set velocity from config
            let velocity_factor = config.velocity_percentage / 100.0;
            vehicle.state.velocity = characteristics.max_velocity * velocity_factor as f64;

            Simulation {
                map: map.clone(),
                vehicle,
                controller: NavigationController::new(&characteristics),
                time: 0.0,
                dt,
                max_time,
                trajectory: Vec::new(),
                distance_threshold: 25.0,
                angle_threshold: 2f64.to_radians(),
                velocity_threshold: characteristics.max_velocity + 5.0,
            }
        })
        .collect();

    println!("Simulando {} vehículos:", simulations.len());
    for (i, sim) in simulations.iter().enumerate() {
        println!("  {}. {} - Inicio: ({:.1}, {:.1}) @ {:.1}°",
            i + 1,
            sim.vehicle.vehicle_type.name(),
            sim.vehicle.state.position.x,
            sim.vehicle.state.position.y,
            sim.vehicle.state.angle.to_degrees()
        );
    }
    println!("\nObjetivo: (500.0, 700.0) @ 90°\n");
    println!("Ejecutando simulación (dt={:.3}s, tiempo_max={:.1}s)...\n", dt, max_time);

    // Run all simulations in parallel
    let mut time = 0.0;
    let mut all_arrived = false;

    while time < max_time && !all_arrived {
        // Update each vehicle
        for sim in &mut simulations {
            if !sim.vehicle.has_arrived {
                sim.step();
            }
        }

        time += dt;

        // Check if all have arrived
        all_arrived = simulations.iter().all(|s| s.vehicle.has_arrived);
    }

    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║            SIMULACIÓN COMPLETADA                      ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Collect results
    let mut vehicle_results = Vec::new();

    for (i, sim) in simulations.into_iter().enumerate() {
        println!("Vehículo {}: {}", i + 1, sim.vehicle.vehicle_type.name());

        // Calculate metrics
        let success = sim.vehicle.has_arrived;
        let arrival_time = if success { Some(sim.time) } else { None };
        let distance_traveled = sim.vehicle.distance_traveled;

        let final_distance = if !sim.trajectory.is_empty() {
            sim.trajectory.last().unwrap().distance_to_target
        } else {
            f64::MAX
        };

        let final_angle_error = if !sim.trajectory.is_empty() {
            (90.0 - sim.trajectory.last().unwrap().angle).abs()
        } else {
            f64::MAX
        };

        println!("  Éxito: {} {}", if success { "SÍ" } else { "NO" }, if success { "✓" } else { "✗" });
        if let Some(t) = arrival_time {
            println!("  Tiempo de Llegada: {:.2}s", t);
        }
        println!("  Distancia Recorrida: {:.2} unidades", distance_traveled);
        println!("  Distancia Final: {:.2} unidades", final_distance);
        println!("  Error Angular Final: {:.2}°\n", final_angle_error);

        let vehicle_result = VehicleResult {
            vehicle_type: sim.vehicle.vehicle_type.name().to_string(),
            trajectory: sim.trajectory,
            metrics: examen_parcial::simulation::SimulationMetrics {
                success,
                arrival_time,
                distance_traveled,
                final_distance_to_target: final_distance,
                final_angle_error,
            },
        };

        vehicle_results.push(vehicle_result);
    }

    let multi_result = MultiVehicleSimulationResult {
        vehicles: vehicle_results,
        total_simulation_time: time,
    };

    // Save to file
    let json_output = serde_json::to_string_pretty(&multi_result).unwrap();
    fs::create_dir_all("output").unwrap();
    let mut file = fs::File::create("output/trajectory_multi.json").unwrap();
    file.write_all(json_output.as_bytes()).unwrap();
    println!("✓ Trayectoria multi-vehículo exportada a: output/trajectory_multi.json\n");

    multi_result
}

struct Visualizer {
    vehicles: Vec<VehicleResult>,
    selected_vehicle: usize,
    current_index: usize,
    is_playing: bool,
    playback_speed: f32,
    time_accumulator: f32,
    map_width: f32,
    map_height: f32,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    // Graph data for selected vehicle
    distance_history: Vec<f32>,
    angle_error_history: Vec<f32>,
}

impl Visualizer {
    fn new(result: MultiVehicleSimulationResult, map_width: f32, map_height: f32) -> Self {
        // Calculate scale to fit map in window (accounting for sidebar)
        let available_width = WINDOW_WIDTH - SIDEBAR_WIDTH - 2.0 * MAP_PADDING;
        let available_height = WINDOW_HEIGHT - 2.0 * MAP_PADDING - 100.0;

        let scale_x = available_width / map_width;
        let scale_y = available_height / map_height;
        let scale = scale_x.min(scale_y);

        let offset_x = SIDEBAR_WIDTH + MAP_PADDING + (available_width - map_width * scale) / 2.0;
        let offset_y = MAP_PADDING;

        // Initialize graph data for first vehicle
        let distance_history = if !result.vehicles.is_empty() {
            result.vehicles[0].trajectory.iter().map(|p| p.distance_to_target as f32).collect()
        } else {
            Vec::new()
        };

        let angle_error_history: Vec<f32> = if !result.vehicles.is_empty() {
            result.vehicles[0].trajectory.iter()
                .map(|p| ((90.0 - p.angle) as f32).abs())
                .collect()
        } else {
            Vec::new()
        };

        Self {
            vehicles: result.vehicles,
            selected_vehicle: 0,
            current_index: 0,
            is_playing: true,
            playback_speed: 1.0,
            time_accumulator: 0.0,
            map_width,
            map_height,
            scale,
            offset_x,
            offset_y,
            distance_history,
            angle_error_history,
        }
    }

    fn update_graph_data(&mut self) {
        if self.selected_vehicle < self.vehicles.len() {
            let vehicle = &self.vehicles[self.selected_vehicle];
            self.distance_history = vehicle.trajectory.iter().map(|p| p.distance_to_target as f32).collect();
            self.angle_error_history = vehicle.trajectory.iter()
                .map(|p| ((90.0 - p.angle) as f32).abs())
                .collect();
        }
    }

    fn world_to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.offset_x + x * self.scale,
            self.offset_y + (self.map_height - y) * self.scale,
        )
    }

    fn update(&mut self, dt: f32) {
        // Update animation for selected vehicle
        if self.selected_vehicle < self.vehicles.len() {
            let trajectory = &self.vehicles[self.selected_vehicle].trajectory;
            if self.is_playing && self.current_index < trajectory.len() - 1 {
                self.time_accumulator += dt * self.playback_speed;

                // Advance multiple frames if playback_speed is high
                while self.current_index < trajectory.len() - 1 {
                    let current_point = &trajectory[self.current_index];
                    let next_point = &trajectory[self.current_index + 1];
                    let dt_trajectory = (next_point.t - current_point.t) as f32;

                    if self.time_accumulator >= dt_trajectory {
                        self.time_accumulator -= dt_trajectory;
                        self.current_index += 1;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn get_vehicle_color(vehicle_type: &str) -> Color {
        match vehicle_type {
            "Barco" => Color::from_rgba(255, 200, 50, 255),      // Yellow/Gold - Barco (Heavy)
            "Lancha" => Color::from_rgba(100, 255, 100, 255),    // Green - Lancha (Standard)
            "Avión" => Color::from_rgba(100, 150, 255, 255),     // Blue - Avión (Agile)
            _ => Color::from_rgba(200, 200, 200, 255),           // Gray
        }
    }

    fn draw_map(&self) {
        // Draw map boundary
        let (x1, y1) = self.world_to_screen(0.0, 0.0);
        let (x2, y2) = self.world_to_screen(self.map_width, self.map_height);
        draw_rectangle_lines(x1, y2, x2 - x1, y1 - y2, 2.0, WHITE);

        // Draw start zone
        let start_zone_height = self.map_height * 0.08;
        let (_, zone_y) = self.world_to_screen(0.0, start_zone_height);
        draw_rectangle(
            x1,
            y1,
            x2 - x1,
            zone_y - y1,
            Color::from_rgba(50, 100, 50, 80),
        );

        // Draw target (square) - LARGER for better visibility
        let (target_x, target_y) = self.world_to_screen(500.0, 700.0);
        let target_size = 50.0;

        draw_rectangle(
            target_x - target_size / 2.0,
            target_y - target_size / 2.0,
            target_size,
            target_size,
            Color::from_rgba(255, 100, 100, 200),
        );
        draw_rectangle_lines(
            target_x - target_size / 2.0,
            target_y - target_size / 2.0,
            target_size,
            target_size,
            3.0,
            RED,
        );

        // Draw required angle indicator - LARGER
        let arrow_len = 35.0;
        draw_line(target_x, target_y, target_x, target_y - arrow_len, 4.0,
            Color::from_rgba(255, 200, 0, 255));
        draw_line(target_x, target_y - arrow_len, target_x - 7.0, target_y - arrow_len + 12.0, 4.0,
            Color::from_rgba(255, 200, 0, 255));
        draw_line(target_x, target_y - arrow_len, target_x + 7.0, target_y - arrow_len + 12.0, 4.0,
            Color::from_rgba(255, 200, 0, 255));

        draw_text("TARGET", target_x - 35.0, target_y + 45.0, 22.0, WHITE);
        draw_text("90°", target_x - 15.0, target_y - arrow_len - 12.0, 20.0,
            Color::from_rgba(255, 200, 0, 255));

        // Draw all vehicle trajectories
        for (idx, vehicle) in self.vehicles.iter().enumerate() {
            let is_selected = idx == self.selected_vehicle;
            let max_idx = if is_selected { self.current_index } else { vehicle.trajectory.len() - 1 };

            let base_color = Self::get_vehicle_color(&vehicle.vehicle_type);
            let alpha_multiplier = if is_selected { 1.0 } else { 0.3 };

            for i in 0..max_idx.min(vehicle.trajectory.len() - 1) {
                let p1 = &vehicle.trajectory[i];
                let p2 = &vehicle.trajectory[i + 1];

                let (x1, y1) = self.world_to_screen(p1.x as f32, p1.y as f32);
                let (x2, y2) = self.world_to_screen(p2.x as f32, p2.y as f32);

                let alpha = if is_selected {
                    let progress = i as f32 / max_idx as f32;
                    ((progress * 200.0 + 55.0) * alpha_multiplier) as u8
                } else {
                    (80.0 * alpha_multiplier) as u8
                };

                let line_color = Color::from_rgba(
                    (base_color.r * 255.0) as u8,
                    (base_color.g * 255.0) as u8,
                    (base_color.b * 255.0) as u8,
                    alpha
                );
                let line_width = if is_selected { 4.0 } else { 2.5 };
                draw_line(x1, y1, x2, y2, line_width, line_color);
            }
        }

        // Draw all vehicles at current position
        for (idx, vehicle) in self.vehicles.iter().enumerate() {
            let is_selected = idx == self.selected_vehicle;
            let traj_idx = if is_selected {
                self.current_index.min(vehicle.trajectory.len() - 1)
            } else {
                vehicle.trajectory.len() - 1  // Show final position for non-selected
            };

            if traj_idx < vehicle.trajectory.len() {
                let current = &vehicle.trajectory[traj_idx];
                let (vx, vy) = self.world_to_screen(current.x as f32, current.y as f32);

                let vehicle_color = Self::get_vehicle_color(&vehicle.vehicle_type);

                if is_selected {
                    // Vehicle body (pulsing effect for selected) - LARGER
                    let pulse = ((current.t * 2.0).sin() * 0.15 + 1.0) as f32;
                    draw_circle(vx, vy, 12.0 * pulse, vehicle_color);
                    draw_circle_lines(vx, vy, 15.0, 2.5, Color::from_rgba(255, 255, 255, 150));
                } else {
                    // Static smaller circle for non-selected - LARGER
                    let dimmed_color = Color::from_rgba(
                        (vehicle_color.r * 255.0) as u8,
                        (vehicle_color.g * 255.0) as u8,
                        (vehicle_color.b * 255.0) as u8,
                        180
                    );
                    draw_circle(vx, vy, 9.0, dimmed_color);
                }

                // Direction indicator - LARGER
                let angle_rad = (current.angle as f32).to_radians();
                let dir_length = if is_selected { 28.0 } else { 22.0 };
                let dx = angle_rad.cos() * dir_length;
                let dy = -angle_rad.sin() * dir_length;
                let arrow_color = if is_selected { RED } else { Color::from_rgba(200, 100, 100, 180) };
                draw_line(vx, vy, vx + dx, vy + dy, 3.5, arrow_color);
                draw_circle(vx + dx, vy + dy, 4.0, arrow_color);
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Simulador de Navegación Difusa - Barco, Lancha y Avión".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

/// Draw loading screen while simulation runs
fn draw_loading_screen(egui_ctx: &egui_macroquad::egui::Context, time: f32) {
    use egui_macroquad::egui;

    egui::CentralPanel::default().show(egui_ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(WINDOW_HEIGHT / 3.0);

            // Animated spinner
            let spinner_size = 80.0;
            ui.add(egui::Spinner::new().size(spinner_size));

            ui.add_space(40.0);

            ui.label(egui::RichText::new("⚙️ Ejecutando Simulación...").size(32.0).strong());
            ui.add_space(20.0);

            ui.label(egui::RichText::new("Por favor espere mientras se simula la navegación").size(18.0).color(egui::Color32::GRAY));
            ui.add_space(10.0);

            // Animated dots
            let dots = match ((time * 2.0) as usize) % 4 {
                0 => "",
                1 => ".",
                2 => "..",
                3 => "...",
                _ => "",
            };
            ui.label(egui::RichText::new(format!("Calculando trayectorias{}", dots)).size(16.0).color(egui::Color32::from_rgb(100, 200, 255)));
        });
    });
}

/// Draw configuration screen - returns true if simulation should start
fn draw_config_screen(egui_ctx: &egui_macroquad::egui::Context, configs: &mut [VehicleConfig], map: &Map) -> bool {
    use egui_macroquad::egui;

    let mut start = false;

    egui::CentralPanel::default().show(egui_ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.heading(egui::RichText::new("⚙️ Configuración de Simulación").size(28.0));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Configure los parámetros iniciales de cada vehículo").size(16.0));
            ui.label(egui::RichText::new("(Los valores aleatorios se generan automáticamente al inicio)").size(14.0).color(egui::Color32::GRAY));
            ui.add_space(30.0);
        });

        ui.separator();
        ui.add_space(20.0);

        // Vehicle configurations
        for (idx, config) in configs.iter_mut().enumerate() {
            let vehicle_name = config.vehicle_type.name().to_string();
            let color = Visualizer::get_vehicle_color(&vehicle_name);
            let egui_color = egui::Color32::from_rgb(
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8
            );

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("🚢 {} - {}", idx + 1, &vehicle_name))
                        .size(20.0)
                        .strong()
                        .color(egui_color));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("🎲 Aleatorizar").size(14.0))
                            .min_size(egui::Vec2::new(130.0, 30.0))).clicked() {
                            config.randomize(map);
                        }
                    });
                });

                // Show vehicle characteristics
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    use examen_parcial::vehicle::create_vehicle_preset;
                    let characteristics = create_vehicle_preset(config.vehicle_type);

                    ui.label(egui::RichText::new(format!(
                        "⚙️ Maniobrabilidad: {:.0}°/s | Vel. Máx: {:.0} u/s",
                        characteristics.maneuverability.to_degrees(),
                        characteristics.max_velocity
                    )).size(13.0).color(egui::Color32::from_gray(180)));
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Posición X:").size(15.0));
                    ui.add(egui::DragValue::new(&mut config.position_x)
                        .speed(1.0)
                        .range(0.0..=1000.0)
                        .suffix(" u"));

                    ui.add_space(20.0);

                    ui.label(egui::RichText::new("Posición Y:").size(15.0));
                    ui.add(egui::DragValue::new(&mut config.position_y)
                        .speed(1.0)
                        .range(0.0..=64.0)
                        .suffix(" u"));
                });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Ángulo:").size(15.0));
                    ui.add(egui::Slider::new(&mut config.angle_degrees, 0.0..=180.0)
                        .suffix("°")
                        .text(""));

                    ui.label(egui::RichText::new(format!("{:.1}°", config.angle_degrees)).size(14.0));
                });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Velocidad:").size(15.0));
                    ui.add(egui::Slider::new(&mut config.velocity_percentage, 5.0..=15.0)
                        .suffix("%")
                        .text(""));

                    ui.label(egui::RichText::new(format!("{:.1}% de velocidad máxima", config.velocity_percentage)).size(14.0));
                });
            });

            ui.add_space(15.0);
        }

        ui.add_space(30.0);
        ui.separator();
        ui.add_space(20.0);

        // Start simulation button
        ui.vertical_centered(|ui| {
            if ui.add(egui::Button::new(egui::RichText::new("▶ Iniciar Simulación").size(22.0))
                .min_size(egui::Vec2::new(300.0, 60.0))
                .fill(egui::Color32::from_rgb(50, 150, 50))).clicked() {
                start = true;
            }

            ui.add_space(10.0);

            if ui.add(egui::Button::new(egui::RichText::new("🎲 Aleatorizar Todos").size(18.0))
                .min_size(egui::Vec2::new(250.0, 45.0))).clicked() {
                for config in configs.iter_mut() {
                    config.randomize(map);
                }
            }
        });
    });

    start
}

#[macroquad::main(window_conf)]
async fn main() {
    // Create map for initial random values
    let map = Map::new(1000.0, 800.0, 500.0, 700.0);

    // Initialize configurations with random values
    let mut configs = vec![
        VehicleConfig::new_random(VehicleType::Heavy, &map),
        VehicleConfig::new_random(VehicleType::Standard, &map),
        VehicleConfig::new_random(VehicleType::Agile, &map),
    ];

    let mut app_state = AppState::Configuration;
    let mut visualizer: Option<Visualizer> = None;
    let mut loading_start_time: f32 = 0.0;
    let mut simulation_triggered = false;

    loop {
        match app_state {
            AppState::Configuration => {
                // Configuration screen
                clear_background(Color::from_rgba(20, 20, 30, 255));

                let mut start_simulation = false;

                egui_macroquad::ui(|egui_ctx| {
                    start_simulation = draw_config_screen(egui_ctx, &mut configs, &map);
                });

                egui_macroquad::draw();

                if start_simulation {
                    app_state = AppState::RunningSimulation;
                    loading_start_time = get_time() as f32;
                    simulation_triggered = false;
                }
            }

            AppState::RunningSimulation => {
                // Loading screen
                clear_background(Color::from_rgba(20, 20, 30, 255));

                let elapsed = get_time() as f32 - loading_start_time;

                egui_macroquad::ui(|egui_ctx| {
                    draw_loading_screen(egui_ctx, elapsed);
                });

                egui_macroquad::draw();

                // Wait one frame to show the loading screen, then run simulation
                if !simulation_triggered {
                    simulation_triggered = true;
                } else {
                    // Run simulation
                    println!("\nIniciando simulación de navegación...\n");
                    let result = run_simulation(&configs);

                    println!("\n✓ Simulación completada. Iniciando visualización...\n");

                    visualizer = Some(Visualizer::new(result, 1000.0, 800.0));
                    app_state = AppState::Visualization;
                }
            }

            AppState::Visualization => {
                // Visualization screen
                if let Some(ref mut viz) = visualizer {
                    let dt = get_frame_time();

                    // Update
                    viz.update(dt);

                    // Draw
                    clear_background(Color::from_rgba(20, 20, 30, 255));

                    // egui UI
                    egui_macroquad::ui(|egui_ctx| {
                        draw_sidebar(egui_ctx, viz);
                    });

                    // Map visualization
                    viz.draw_map();

                    // Render egui
                    egui_macroquad::draw();
                }
            }
        }

        next_frame().await;
    }
}

fn draw_sidebar(egui_ctx: &egui_macroquad::egui::Context, viz: &mut Visualizer) {
    use egui_macroquad::egui;
    egui::SidePanel::left("control_panel")
        .exact_width(SIDEBAR_WIDTH)
        .resizable(false)
        .show(egui_ctx, |ui| {
            ui.heading(egui::RichText::new("🚢 Visualizador de Navegación").size(20.0));
            ui.separator();

            // === VEHICLE SELECTOR ===
            ui.group(|ui| {
                ui.label(egui::RichText::new("🎯 Seleccionar Vehículo").strong().size(16.0));
                ui.add_space(8.0);

                let vehicle_count = viz.vehicles.len();
                let mut new_selection: Option<usize> = None;

                ui.horizontal(|ui| {
                    for idx in 0..vehicle_count {
                        let is_selected = idx == viz.selected_vehicle;
                        let vehicle_type = &viz.vehicles[idx].vehicle_type;
                        let color = Visualizer::get_vehicle_color(vehicle_type);
                        let button_color = egui::Color32::from_rgb(
                            (color.r * 255.0) as u8,
                            (color.g * 255.0) as u8,
                            (color.b * 255.0) as u8
                        );

                        let button_text = if is_selected {
                            egui::RichText::new(vehicle_type).strong().size(16.0)
                        } else {
                            egui::RichText::new(vehicle_type).size(15.0)
                        };

                        let button = egui::Button::new(button_text)
                            .fill(if is_selected { button_color } else { egui::Color32::from_gray(60) })
                            .min_size(egui::Vec2::new(110.0, 35.0));

                        if ui.add(button).clicked() && viz.selected_vehicle != idx {
                            new_selection = Some(idx);
                        }
                    }
                });

                // Update selection after the borrow ends
                if let Some(idx) = new_selection {
                    viz.selected_vehicle = idx;
                    viz.current_index = 0;
                    viz.time_accumulator = 0.0;
                    viz.update_graph_data();
                }
            });

            ui.add_space(12.0);

            // === PLAYBACK CONTROLS ===
            ui.group(|ui| {
                ui.label(egui::RichText::new("⏯ Controles de Reproducción").strong().size(16.0));
                ui.add_space(8.0);

                // Play/Pause button
                let button_text = if viz.is_playing { "⏸ Pausar" } else { "▶ Reproducir" };
                if ui.add(egui::Button::new(egui::RichText::new(button_text).size(15.0))
                    .min_size(egui::Vec2::new(150.0, 35.0))).clicked() {
                    viz.is_playing = !viz.is_playing;
                }

                ui.add_space(8.0);

                // Speed slider
                ui.label(egui::RichText::new("Velocidad:").size(14.0));
                ui.add(egui::Slider::new(&mut viz.playback_speed, 0.1..=100.0)
                    .text("x")
                    .logarithmic(true));

                // Reset button
                if ui.add(egui::Button::new(egui::RichText::new("🔄 Reiniciar").size(15.0))
                    .min_size(egui::Vec2::new(150.0, 35.0))).clicked() {
                    viz.current_index = 0;
                    viz.time_accumulator = 0.0;
                }
            });

            ui.add_space(12.0);

            // === PROGRESS ===
            ui.group(|ui| {
                ui.label(egui::RichText::new("📊 Progreso").strong().size(16.0));
                ui.add_space(8.0);

                if viz.selected_vehicle < viz.vehicles.len() {
                    let selected = &viz.vehicles[viz.selected_vehicle];
                    let progress = viz.current_index as f32 / selected.trajectory.len() as f32;

                    let progress_bar = egui::ProgressBar::new(progress)
                        .text(egui::RichText::new(format!("{:.1}%", progress * 100.0)).size(14.0))
                        .animate(viz.is_playing);
                    ui.add(progress_bar);

                    ui.label(egui::RichText::new(format!("Fotograma: {}/{}", viz.current_index, selected.trajectory.len())).size(13.0));

                    if viz.current_index < selected.trajectory.len() {
                        let current = &selected.trajectory[viz.current_index];
                        ui.label(egui::RichText::new(format!("Tiempo: {:.2}s", current.t)).size(13.0));
                    }
                }
            });

            ui.add_space(12.0);

            // === REALTIME STATS ===
            if viz.selected_vehicle < viz.vehicles.len() {
                let selected = &viz.vehicles[viz.selected_vehicle];
                if viz.current_index < selected.trajectory.len() {
                    let current = &selected.trajectory[viz.current_index];

                    ui.group(|ui| {
                        ui.label(egui::RichText::new("📈 Estado Actual").strong().size(16.0));
                        ui.add_space(8.0);

                        ui.label(egui::RichText::new(format!("🧭 Posición: ({:.1}, {:.1})", current.x, current.y)).size(13.0));
                        ui.label(egui::RichText::new(format!("📐 Ángulo: {:.1}°", current.angle)).size(13.0));
                        ui.label(egui::RichText::new(format!("🎯 Distancia al Objetivo: {:.1} unidades", current.distance_to_target)).size(13.0));
                        ui.label(egui::RichText::new(format!("⚡ Velocidad: {:.1} u/s", current.velocity)).size(13.0));

                        let angle_error = (90.0 - current.angle).abs();
                        let error_color = if angle_error < 10.0 {
                            egui::Color32::GREEN
                        } else if angle_error < 40.0 {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::RED
                        };

                        ui.label(egui::RichText::new(format!("Δ Ángulo desde 90°: {:.1}°", angle_error))
                            .color(error_color)
                            .size(13.0));
                    });
                }
            }

            ui.add_space(12.0);

            // === GRAPHS ===
            ui.group(|ui| {
                ui.label(egui::RichText::new("📉 Gráficas de Métricas").strong().size(16.0));
                ui.add_space(8.0);

                // Distance graph
                ui.label(egui::RichText::new("Distancia al Objetivo:").size(13.0));
                draw_mini_graph(ui, &viz.distance_history, viz.current_index, "unid",
                    egui::Color32::from_rgb(100, 200, 255));

                ui.add_space(10.0);

                // Angle error graph
                ui.label(egui::RichText::new("Error de Ángulo desde 90°:").size(13.0));
                draw_mini_graph(ui, &viz.angle_error_history, viz.current_index, "°",
                    egui::Color32::from_rgb(255, 200, 100));
            });

            ui.add_space(12.0);

            // === FINAL METRICS ===
            if viz.selected_vehicle < viz.vehicles.len() {
                let selected = &viz.vehicles[viz.selected_vehicle];
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🏁 Resultados Finales").strong().size(16.0));
                    ui.add_space(8.0);

                    let success_icon = if selected.metrics.success { "✅" } else { "❌" };
                    ui.label(egui::RichText::new(format!("{} Estado: {}", success_icon,
                        if selected.metrics.success { "Llegó" } else { "No llegó" })).size(13.0));

                    if let Some(time) = selected.metrics.arrival_time {
                        ui.label(egui::RichText::new(format!("⏱ Tiempo de Llegada: {:.2}s", time)).size(13.0));
                    }

                    ui.label(egui::RichText::new(format!("📏 Distancia Recorrida: {:.1} unid", selected.metrics.distance_traveled)).size(13.0));
                    ui.label(egui::RichText::new(format!("🎯 Distancia Final: {:.1} unid", selected.metrics.final_distance_to_target)).size(13.0));
                    ui.label(egui::RichText::new(format!("📐 Error Angular Final: {:.1}°", selected.metrics.final_angle_error)).size(13.0));
                });
            }

            ui.add_space(12.0);

            // === COMPARISON TABLE ===
            ui.group(|ui| {
                ui.label(egui::RichText::new("📊 Comparación de Vehículos").strong().size(16.0));
                ui.add_space(8.0);

                use egui_macroquad::egui::Grid;
                Grid::new("comparison_grid")
                    .striped(true)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        // Header
                        ui.label(egui::RichText::new("Vehículo").strong().size(13.0));
                        ui.label(egui::RichText::new("Estado").strong().size(13.0));
                        ui.label(egui::RichText::new("Tiempo").strong().size(13.0));
                        ui.label(egui::RichText::new("Δ Ángulo").strong().size(13.0));
                        ui.end_row();

                        // Data rows
                        for vehicle in &viz.vehicles {
                            let color = Visualizer::get_vehicle_color(&vehicle.vehicle_type);
                            let egui_color = egui::Color32::from_rgb(
                                (color.r * 255.0) as u8,
                                (color.g * 255.0) as u8,
                                (color.b * 255.0) as u8
                            );

                            ui.label(egui::RichText::new(&vehicle.vehicle_type).color(egui_color).size(12.0));

                            let status = if vehicle.metrics.success { "✅" } else { "❌" };
                            ui.label(egui::RichText::new(status).size(12.0));

                            if let Some(time) = vehicle.metrics.arrival_time {
                                ui.label(egui::RichText::new(format!("{:.1}s", time)).size(12.0));
                            } else {
                                ui.label(egui::RichText::new("N/A").size(12.0));
                            }

                            let angle_color = if vehicle.metrics.final_angle_error < 2.0 {
                                egui::Color32::GREEN
                            } else if vehicle.metrics.final_angle_error < 10.0 {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::RED
                            };

                            ui.label(egui::RichText::new(format!("{:.1}°", vehicle.metrics.final_angle_error))
                                .color(angle_color)
                                .size(12.0));

                            ui.end_row();
                        }
                    });
            });

            ui.add_space(12.0);

            // === KEYBINDINGS ===
            ui.group(|ui| {
                ui.label(egui::RichText::new("⌨ Atajos de Teclado").strong().size(14.0));
                ui.add_space(5.0);
                ui.label(egui::RichText::new("ESPACIO: Reproducir/Pausar").size(12.0));
                ui.label(egui::RichText::new("←/→: Velocidad").size(12.0));
                ui.label(egui::RichText::new("R: Reiniciar").size(12.0));
            });
        });
}

fn draw_mini_graph(ui: &mut egui_macroquad::egui::Ui, data: &[f32], current_idx: usize, unit: &str, color: egui_macroquad::egui::Color32) {
    use egui_macroquad::egui;

    // Simple canvas-based graph
    let graph_height = 80.0;
    let (response, painter) = ui.allocate_painter(egui::Vec2::new(ui.available_width(), graph_height), egui::Sense::hover());
    let rect = response.rect;

    if current_idx > 0 && !data.is_empty() {
        // Find min/max for scaling
        let visible_data: Vec<f32> = data.iter().take(current_idx + 1).cloned().collect();
        let max_val = visible_data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min_val = visible_data.iter().cloned().fold(f32::INFINITY, f32::min);
        let range = (max_val - min_val).max(0.1);

        // Draw background
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

        // Draw grid lines
        for i in 0..5 {
            let y = rect.top() + (i as f32 / 4.0) * rect.height();
            painter.line_segment(
                [egui::Pos2::new(rect.left(), y), egui::Pos2::new(rect.right(), y)],
                egui::Stroke::new(0.5, egui::Color32::from_gray(50))
            );
        }

        // Draw line graph
        let mut points: Vec<egui::Pos2> = Vec::new();
        for (i, &value) in visible_data.iter().enumerate() {
            let x = rect.left() + (i as f32 / (visible_data.len() - 1) as f32) * rect.width();
            let normalized = (value - min_val) / range;
            let y = rect.bottom() - normalized * rect.height();
            points.push(egui::Pos2::new(x, y));
        }

        // Draw the line
        for i in 0..points.len().saturating_sub(1) {
            painter.line_segment(
                [points[i], points[i + 1]],
                egui::Stroke::new(2.0, color)
            );
        }

        // Draw current value marker
        if current_idx < points.len() {
            painter.circle_filled(points[current_idx], 3.0, egui::Color32::WHITE);
        }
    }

    // Current value label
    if current_idx < data.len() {
        ui.label(egui::RichText::new(format!("Current: {:.1} {}", data[current_idx], unit))
            .small()
            .color(egui::Color32::LIGHT_GRAY));
    }
}
