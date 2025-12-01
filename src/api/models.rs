// API models for requests and responses
use serde::{Deserialize, Serialize};
use crate::vehicle::VehicleType;
use crate::simulation::{SimulationMetrics, TrajectoryPoint};

// ============================================================================
// REQUEST MODELS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SimulationRequest {
    /// Vehicle types to simulate (Heavy, Standard, Agile)
    #[serde(default = "default_vehicle_types")]
    pub vehicle_types: Vec<String>,

    /// Time step in seconds (default: 0.05)
    #[serde(default = "default_dt")]
    pub dt: f64,

    /// Maximum simulation time in seconds (default: 600.0)
    #[serde(default = "default_max_time")]
    pub max_time: f64,

    /// Map width (default: 1000.0)
    #[serde(default = "default_map_width")]
    pub map_width: f64,

    /// Map height (default: 800.0)
    #[serde(default = "default_map_height")]
    pub map_height: f64,

    /// Target X coordinate (default: 500.0)
    #[serde(default = "default_target_x")]
    pub target_x: f64,

    /// Target Y coordinate (default: 700.0)
    #[serde(default = "default_target_y")]
    pub target_y: f64,
}

fn default_vehicle_types() -> Vec<String> {
    vec!["Heavy".to_string(), "Standard".to_string(), "Agile".to_string()]
}

fn default_dt() -> f64 { 0.05 }
fn default_max_time() -> f64 { 600.0 }
fn default_map_width() -> f64 { 1000.0 }
fn default_map_height() -> f64 { 800.0 }
fn default_target_x() -> f64 { 500.0 }
fn default_target_y() -> f64 { 700.0 }

#[derive(Debug, Deserialize)]
pub struct BenchmarkRequest {
    /// Number of iterations to run (default: 30)
    #[serde(default = "default_iterations")]
    pub iterations: usize,

    /// Vehicle types to benchmark (default: all types)
    #[serde(default = "default_vehicle_types")]
    pub vehicle_types: Vec<String>,

    /// Number of threads to use (default: half of available cores)
    pub threads: Option<usize>,

    /// Time step in seconds (default: 0.05)
    #[serde(default = "default_dt")]
    pub dt: f64,

    /// Maximum simulation time in seconds (default: 600.0)
    #[serde(default = "default_max_time")]
    pub max_time: f64,
}

fn default_iterations() -> usize { 30 }

// ============================================================================
// RESPONSE MODELS
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SimulationResponse {
    pub success: bool,
    pub vehicles: Vec<VehicleSimulationResult>,
    pub total_simulation_time: f64,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct VehicleSimulationResult {
    pub vehicle_type: String,
    pub trajectory: Vec<TrajectoryPoint>,
    pub metrics: SimulationMetrics,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkResponse {
    pub success: bool,
    pub num_iterations: usize,
    pub aggregate_stats: Vec<AggregateStats>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct AggregateStats {
    pub vehicle_type: String,
    pub total_runs: usize,
    pub successes: usize,
    pub success_rate: f64,
    pub avg_arrival_time: f64,
    pub std_arrival_time: f64,
    pub min_arrival_time: f64,
    pub max_arrival_time: f64,
    pub avg_distance_traveled: f64,
    pub std_distance_traveled: f64,
    pub avg_final_distance: f64,
    pub avg_final_angle_error: f64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub message: String,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

impl SimulationRequest {
    pub fn parse_vehicle_types(&self) -> Result<Vec<VehicleType>, String> {
        self.vehicle_types
            .iter()
            .map(|s| match s.to_lowercase().as_str() {
                "heavy" => Ok(VehicleType::Heavy),
                "standard" => Ok(VehicleType::Standard),
                "agile" => Ok(VehicleType::Agile),
                _ => Err(format!("Unknown vehicle type: {}. Valid types: Heavy, Standard, Agile", s)),
            })
            .collect()
    }
}

impl BenchmarkRequest {
    pub fn parse_vehicle_types(&self) -> Result<Vec<VehicleType>, String> {
        self.vehicle_types
            .iter()
            .map(|s| match s.to_lowercase().as_str() {
                "heavy" => Ok(VehicleType::Heavy),
                "standard" => Ok(VehicleType::Standard),
                "agile" => Ok(VehicleType::Agile),
                _ => Err(format!("Unknown vehicle type: {}. Valid types: Heavy, Standard, Agile", s)),
            })
            .collect()
    }
}
