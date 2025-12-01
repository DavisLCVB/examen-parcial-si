// API handlers for REST endpoints
use shuttle_axum::axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::map::Map;
use crate::simulation::Simulation;
use super::models::*;

// ============================================================================
// ERROR HANDLING
// ============================================================================

pub enum ApiError {
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse {
            error: status.to_string(),
            details: Some(message),
        });

        (status, body).into_response()
    }
}

// ============================================================================
// HEALTH CHECK
// ============================================================================

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        message: "Fuzzy Navigation System API is running".to_string(),
    })
}

// ============================================================================
// SIMULATION ENDPOINT
// ============================================================================

pub async fn run_simulation(
    Json(request): Json<SimulationRequest>,
) -> Result<Json<SimulationResponse>, ApiError> {
    // Parse vehicle types
    let vehicle_types = request.parse_vehicle_types()
        .map_err(|e| ApiError::BadRequest(e))?;

    if vehicle_types.is_empty() {
        return Err(ApiError::BadRequest(
            "At least one vehicle type must be specified".to_string()
        ));
    }

    // Create map
    let map = Map::new(
        request.map_width,
        request.map_height,
        request.target_x,
        request.target_y,
    );

    // Run simulations in blocking task to avoid blocking async runtime
    let vehicles_result = tokio::task::spawn_blocking(move || {
        let mut simulations: Vec<Simulation> = vehicle_types
            .iter()
            .map(|&vtype| Simulation::new(map.clone(), vtype, request.dt, request.max_time))
            .collect();

        let mut time = 0.0;
        let mut all_arrived = false;

        while time < request.max_time && !all_arrived {
            for sim in &mut simulations {
                if !sim.vehicle.has_arrived {
                    sim.step();
                }
            }

            time += request.dt;
            all_arrived = simulations.iter().all(|s| s.vehicle.has_arrived);
        }

        // Collect results
        let vehicle_results: Vec<VehicleSimulationResult> = simulations
            .into_iter()
            .map(|sim| {
                let success = sim.vehicle.has_arrived;
                let arrival_time = if success { Some(sim.vehicle.time_elapsed) } else { None };

                // Handle empty trajectory case
                let (final_distance, final_angle_error) = if let Some(final_point) = sim.trajectory.last() {
                    (final_point.distance_to_target, (90.0 - final_point.angle).abs())
                } else {
                    // If no trajectory points, calculate from current vehicle state
                    let dx = sim.vehicle.state.position.x - sim.map.target.position.x;
                    let dy = sim.vehicle.state.position.y - sim.map.target.position.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let angle_error = (90.0 - sim.vehicle.state.angle.to_degrees()).abs();
                    (dist, angle_error)
                };

                let mut distance_traveled = 0.0;
                for j in 1..sim.trajectory.len() {
                    let p1 = &sim.trajectory[j - 1];
                    let p2 = &sim.trajectory[j];
                    let dx = p2.x - p1.x;
                    let dy = p2.y - p1.y;
                    distance_traveled += (dx * dx + dy * dy).sqrt();
                }

                let metrics = crate::simulation::SimulationMetrics {
                    success,
                    arrival_time,
                    distance_traveled,
                    final_angle_error,
                    final_distance_to_target: final_distance,
                };

                VehicleSimulationResult {
                    vehicle_type: sim.vehicle.vehicle_type.name().to_string(),
                    trajectory: sim.trajectory.clone(),
                    metrics,
                }
            })
            .collect();

        (vehicle_results, time)
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Simulation task failed: {}", e)))?;

    let (vehicles, total_time) = vehicles_result;

    let success_count = vehicles.iter().filter(|v| v.metrics.success).count();
    let message = format!(
        "Simulation completed: {}/{} vehicles arrived successfully",
        success_count,
        vehicles.len()
    );

    Ok(Json(SimulationResponse {
        success: true,
        vehicles,
        total_simulation_time: total_time,
        message,
    }))
}

// ============================================================================
// BENCHMARK ENDPOINT
// ============================================================================

#[derive(Clone)]
struct VehicleMetrics {
    vehicle_type: String,
    success: bool,
    arrival_time: Option<f64>,
    distance_traveled: f64,
    final_distance: f64,
    final_angle_error: f64,
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

pub async fn run_benchmark(
    Json(request): Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkResponse>, ApiError> {
    // Parse vehicle types
    let vehicle_types = request.parse_vehicle_types()
        .map_err(|e| ApiError::BadRequest(e))?;

    if vehicle_types.is_empty() {
        return Err(ApiError::BadRequest(
            "At least one vehicle type must be specified".to_string()
        ));
    }

    if request.iterations == 0 {
        return Err(ApiError::BadRequest(
            "Number of iterations must be greater than 0".to_string()
        ));
    }

    // Store count before moving vehicle_types
    let num_vehicle_types = vehicle_types.len();

    // Run benchmark in blocking task
    let aggregate_stats = tokio::task::spawn_blocking(move || {
        // Configure rayon thread pool
        let available_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let threads_to_use = request.threads.unwrap_or(available_threads / 2);

        rayon::ThreadPoolBuilder::new()
            .num_threads(threads_to_use)
            .build_global()
            .ok();

        let map = Map::new(1000.0, 800.0, 500.0, 700.0);

        let completed = Arc::new(AtomicUsize::new(0));
        let completed_clone = Arc::clone(&completed);

        // Run iterations in parallel
        let all_results: Vec<Vec<VehicleMetrics>> = (0..request.iterations)
            .into_par_iter()
            .map(|_| {
                let iteration_vehicles: Vec<VehicleMetrics> = vehicle_types
                    .iter()
                    .map(|&vtype| {
                        let mut sim = Simulation::new(map.clone(), vtype, request.dt, request.max_time);

                        while sim.time < request.max_time && !sim.vehicle.has_arrived {
                            sim.step();
                        }

                        let success = sim.vehicle.has_arrived;
                        let arrival_time = if success { Some(sim.vehicle.time_elapsed) } else { None };

                        // Handle empty trajectory case
                        let (final_distance, final_angle_error) = if let Some(final_point) = sim.trajectory.last() {
                            (final_point.distance_to_target, (90.0 - final_point.angle).abs())
                        } else {
                            // If no trajectory points, calculate from current vehicle state
                            let dx = sim.vehicle.state.position.x - sim.map.target.position.x;
                            let dy = sim.vehicle.state.position.y - sim.map.target.position.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            let angle_error = (90.0 - sim.vehicle.state.angle.to_degrees()).abs();
                            (dist, angle_error)
                        };

                        let mut distance_traveled = 0.0;
                        for j in 1..sim.trajectory.len() {
                            let p1 = &sim.trajectory[j - 1];
                            let p2 = &sim.trajectory[j];
                            let dx = p2.x - p1.x;
                            let dy = p2.y - p1.y;
                            distance_traveled += (dx * dx + dy * dy).sqrt();
                        }

                        VehicleMetrics {
                            vehicle_type: vtype.name().to_string(),
                            success,
                            arrival_time,
                            distance_traveled,
                            final_distance,
                            final_angle_error,
                        }
                    })
                    .collect();

                completed_clone.fetch_add(1, Ordering::Relaxed);
                iteration_vehicles
            })
            .collect();

        // Reorganize results by vehicle type
        let mut all_metrics: Vec<Vec<VehicleMetrics>> = vec![Vec::new(); vehicle_types.len()];
        for iteration_result in &all_results {
            for (idx, metrics) in iteration_result.iter().enumerate() {
                all_metrics[idx].push(metrics.clone());
            }
        }

        // Calculate aggregate statistics
        let mut stats: Vec<AggregateStats> = Vec::new();

        for (idx, vtype) in vehicle_types.iter().enumerate() {
            let metrics = &all_metrics[idx];
            let successes = metrics.iter().filter(|m| m.success).count();
            let success_rate = successes as f64 / request.iterations as f64 * 100.0;

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

            stats.push(AggregateStats {
                vehicle_type: vtype.name().to_string(),
                total_runs: request.iterations,
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

        stats
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Benchmark task failed: {}", e)))?;

    let message = format!("Benchmark completed: {} iterations across {} vehicle types",
        request.iterations,
        num_vehicle_types
    );

    Ok(Json(BenchmarkResponse {
        success: true,
        num_iterations: request.iterations,
        aggregate_stats,
        message,
    }))
}
