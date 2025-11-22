// Benchmark: Run multiple simulations to collect metrics for research
//
// Run with: cargo run --bin benchmark -- [num_iterations]
// Example: cargo run --bin benchmark -- 100

use examen_parcial::map::Map;
use examen_parcial::simulation::Simulation;
use examen_parcial::vehicle::VehicleType;
use serde::Serialize;
use std::env;
use std::fs;
use std::io::Write;

#[derive(Serialize, Clone)]
struct VehicleMetrics {
    vehicle_type: String,
    success: bool,
    arrival_time: Option<f64>,
    distance_traveled: f64,
    final_distance: f64,
    final_angle_error: f64,
    initial_x: f64,
    initial_y: f64,
    initial_angle: f64,
}

#[derive(Serialize)]
struct IterationResult {
    iteration: usize,
    vehicles: Vec<VehicleMetrics>,
}

#[derive(Serialize)]
struct AggregateStats {
    vehicle_type: String,
    total_runs: usize,
    successes: usize,
    success_rate: f64,
    avg_arrival_time: f64,
    std_arrival_time: f64,
    min_arrival_time: f64,
    max_arrival_time: f64,
    avg_distance_traveled: f64,
    std_distance_traveled: f64,
    avg_final_distance: f64,
    avg_final_angle_error: f64,
}

#[derive(Serialize)]
struct BenchmarkResult {
    num_iterations: usize,
    dt: f64,
    max_time: f64,
    map_width: f64,
    map_height: f64,
    target_x: f64,
    target_y: f64,
    iterations: Vec<IterationResult>,
    aggregate: Vec<AggregateStats>,
}

fn calculate_stats(values: &[f64]) -> (f64, f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    (mean, std, min, max)
}

fn run_single_simulation(map: &Map, vehicle_type: VehicleType, dt: f64, max_time: f64) -> VehicleMetrics {
    let mut sim = Simulation::new(map.clone(), vehicle_type, dt, max_time);

    let initial_x = sim.vehicle.state.position.x;
    let initial_y = sim.vehicle.state.position.y;
    let initial_angle = sim.vehicle.state.angle.to_degrees();

    // Run simulation
    while sim.time < max_time && !sim.vehicle.has_arrived {
        sim.step();
    }

    let success = sim.vehicle.has_arrived;
    let arrival_time = if success { Some(sim.vehicle.time_elapsed) } else { None };

    let final_point = sim.trajectory.last().unwrap();
    let final_distance = final_point.distance_to_target;
    let final_angle_error = (90.0 - final_point.angle).abs();

    // Calculate distance traveled
    let mut distance_traveled = 0.0;
    for j in 1..sim.trajectory.len() {
        let p1 = &sim.trajectory[j - 1];
        let p2 = &sim.trajectory[j];
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        distance_traveled += (dx * dx + dy * dy).sqrt();
    }

    VehicleMetrics {
        vehicle_type: vehicle_type.name().to_string(),
        success,
        arrival_time,
        distance_traveled,
        final_distance,
        final_angle_error,
        initial_x,
        initial_y,
        initial_angle,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let num_iterations: usize = args.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);

    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║   FUZZY NAVIGATION BENCHMARK                         ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    let map = Map::new(1000.0, 800.0, 500.0, 700.0);
    let dt = 0.05;
    let max_time = 600.0;

    let vehicle_types = vec![
        VehicleType::Heavy,
        VehicleType::Standard,
        VehicleType::Agile,
    ];

    println!("Configuration:");
    println!("  Iterations: {}", num_iterations);
    println!("  Vehicles: Heavy, Standard, Agile");
    println!("  dt: {}s, max_time: {}s", dt, max_time);
    println!("  Target: (500, 700) @ 90 deg\n");

    let mut all_iterations: Vec<IterationResult> = Vec::new();
    let mut all_metrics: Vec<Vec<VehicleMetrics>> = vec![Vec::new(); vehicle_types.len()];

    for i in 0..num_iterations {
        print!("\rRunning iteration {}/{}...", i + 1, num_iterations);
        std::io::stdout().flush().unwrap();

        let mut iteration_vehicles = Vec::new();

        for (idx, &vtype) in vehicle_types.iter().enumerate() {
            let metrics = run_single_simulation(&map, vtype, dt, max_time);
            all_metrics[idx].push(metrics.clone());
            iteration_vehicles.push(metrics);
        }

        all_iterations.push(IterationResult {
            iteration: i + 1,
            vehicles: iteration_vehicles,
        });
    }

    println!("\r\n\n╔══════════════════════════════════════════════════════╗");
    println!("║            BENCHMARK RESULTS                          ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Calculate aggregate statistics
    let mut aggregate_stats: Vec<AggregateStats> = Vec::new();

    for (idx, vtype) in vehicle_types.iter().enumerate() {
        let metrics = &all_metrics[idx];
        let successes = metrics.iter().filter(|m| m.success).count();
        let success_rate = successes as f64 / num_iterations as f64 * 100.0;

        let arrival_times: Vec<f64> = metrics.iter()
            .filter_map(|m| m.arrival_time)
            .collect();
        let (avg_time, std_time, min_time, max_time) = calculate_stats(&arrival_times);

        let distances: Vec<f64> = metrics.iter().map(|m| m.distance_traveled).collect();
        let (avg_dist, std_dist, _, _) = calculate_stats(&distances);

        let final_dists: Vec<f64> = metrics.iter().map(|m| m.final_distance).collect();
        let (avg_final_dist, _, _, _) = calculate_stats(&final_dists);

        let angle_errors: Vec<f64> = metrics.iter().map(|m| m.final_angle_error).collect();
        let (avg_angle_error, _, _, _) = calculate_stats(&angle_errors);

        println!("{}:", vtype.name());
        println!("  Success Rate: {:.1}% ({}/{})", success_rate, successes, num_iterations);
        println!("  Arrival Time: {:.2}s avg (std: {:.2}, min: {:.2}, max: {:.2})",
            avg_time, std_time, min_time, max_time);
        println!("  Distance Traveled: {:.2} avg (std: {:.2})", avg_dist, std_dist);
        println!("  Final Distance: {:.2} avg", avg_final_dist);
        println!("  Final Angle Error: {:.2} deg avg\n", avg_angle_error);

        aggregate_stats.push(AggregateStats {
            vehicle_type: vtype.name().to_string(),
            total_runs: num_iterations,
            successes,
            success_rate,
            avg_arrival_time: avg_time,
            std_arrival_time: std_time,
            min_arrival_time: min_time,
            max_arrival_time: max_time,
            avg_distance_traveled: avg_dist,
            std_distance_traveled: std_dist,
            avg_final_distance: avg_final_dist,
            avg_final_angle_error: avg_angle_error,
        });
    }

    // Export results
    let result = BenchmarkResult {
        num_iterations,
        dt,
        max_time,
        map_width: 1000.0,
        map_height: 800.0,
        target_x: 500.0,
        target_y: 700.0,
        iterations: all_iterations,
        aggregate: aggregate_stats,
    };

    fs::create_dir_all("output").expect("Failed to create output directory");

    let json = serde_json::to_string_pretty(&result).unwrap();
    let filename = format!("output/benchmark_{}iterations.json", num_iterations);
    fs::write(&filename, &json).expect("Failed to write benchmark results");

    // Export CSV for easy analysis
    let csv_filename = format!("output/benchmark_{}iterations.csv", num_iterations);
    let mut csv = String::from("iteration,vehicle_type,success,arrival_time,distance_traveled,final_distance,final_angle_error,initial_x,initial_y,initial_angle\n");

    for iter in &result.iterations {
        for v in &iter.vehicles {
            csv.push_str(&format!(
                "{},{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
                iter.iteration,
                v.vehicle_type,
                v.success,
                v.arrival_time.map(|t| format!("{:.2}", t)).unwrap_or_default(),
                v.distance_traveled,
                v.final_distance,
                v.final_angle_error,
                v.initial_x,
                v.initial_y,
                v.initial_angle
            ));
        }
    }
    fs::write(&csv_filename, &csv).expect("Failed to write CSV");

    // Export aggregate stats CSV
    let agg_csv_filename = format!("output/benchmark_{}iterations_summary.csv", num_iterations);
    let mut agg_csv = String::from("vehicle_type,total_runs,successes,success_rate,avg_arrival_time,std_arrival_time,min_arrival_time,max_arrival_time,avg_distance_traveled,std_distance_traveled,avg_final_distance,avg_final_angle_error\n");

    for stat in &result.aggregate {
        agg_csv.push_str(&format!(
            "{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
            stat.vehicle_type,
            stat.total_runs,
            stat.successes,
            stat.success_rate,
            stat.avg_arrival_time,
            stat.std_arrival_time,
            stat.min_arrival_time,
            stat.max_arrival_time,
            stat.avg_distance_traveled,
            stat.std_distance_traveled,
            stat.avg_final_distance,
            stat.avg_final_angle_error
        ));
    }
    fs::write(&agg_csv_filename, &agg_csv).expect("Failed to write summary CSV");

    println!("Results exported to:");
    println!("  - {} (JSON)", filename);
    println!("  - {} (CSV raw data)", csv_filename);
    println!("  - {} (CSV summary)", agg_csv_filename);
}
