// Simulation module - Main simulation loop and physics engine

use crate::map::{clamp, compute_angular_error_with_arrival, euclidean_distance, normalize_angle, Map, Point};
use crate::navigation::NavigationController;
use crate::vehicle::{create_vehicle_preset, Vehicle, VehicleType};
use serde::{Deserialize, Serialize};

// Conditional printing macro - only prints when CLI feature is enabled
#[cfg(feature = "cli")]
macro_rules! sim_println {
    ($($arg:tt)*) => {
        println!($($arg)*)
    };
}

#[cfg(not(feature = "cli"))]
macro_rules! sim_println {
    ($($arg:tt)*) => {};
}

/// Snapshot of vehicle state at a given time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryPoint {
    pub t: f64,
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub velocity: f64,
    pub distance_to_target: f64,
}

/// Complete simulation result for export
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationResult {
    pub vehicle_type: String,
    pub trajectory: Vec<TrajectoryPoint>,
    pub metrics: SimulationMetrics,
}

/// Performance metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub success: bool,
    pub arrival_time: Option<f64>,
    pub distance_traveled: f64,
    pub final_angle_error: f64,
    pub final_distance_to_target: f64,
}

/// Result for a single vehicle in multi-vehicle simulation
#[derive(Debug, Serialize, Deserialize)]
pub struct VehicleResult {
    pub vehicle_type: String,
    pub trajectory: Vec<TrajectoryPoint>,
    pub metrics: SimulationMetrics,
}

/// Complete multi-vehicle simulation result
#[derive(Debug, Serialize, Deserialize)]
pub struct MultiVehicleSimulationResult {
    pub vehicles: Vec<VehicleResult>,
    pub total_simulation_time: f64,
}

/// Main simulation controller
pub struct Simulation {
    pub map: Map,
    pub vehicle: Vehicle,
    pub controller: NavigationController,
    pub time: f64,
    pub dt: f64,
    pub max_time: f64,
    pub trajectory: Vec<TrajectoryPoint>,

    // Arrival criteria
    pub distance_threshold: f64,
    pub angle_threshold: f64,
    pub velocity_threshold: f64,
}

impl Simulation {
    /// Create a new simulation with a vehicle type
    pub fn new(
        map: Map,
        vehicle_type: VehicleType,
        dt: f64,
        max_time: f64,
    ) -> Self {
        let characteristics = create_vehicle_preset(vehicle_type);
        let initial_pos = map.random_start_position();
        let initial_angle = map.random_start_angle();

        let mut vehicle = Vehicle::new(
            vehicle_type,
            characteristics.clone(),
            initial_pos,
            initial_angle,
        );

        // Set constant velocity at 10% of max speed for high precision 90° arrival (±2°)
        let constant_velocity = characteristics.max_velocity * 0.10;
        vehicle.state.velocity = constant_velocity;

        let controller = NavigationController::new(&characteristics);

        Self {
            map,
            vehicle,
            controller,
            time: 0.0,
            dt,
            max_time,
            trajectory: Vec::new(),
            distance_threshold: 25.0,  // 25 units
            angle_threshold: 2f64.to_radians(),  // ±2° tolerance (88-92°) - STRICT
            velocity_threshold: constant_velocity + 5.0,  // Allow slightly above constant
        }
    }

    /// Execute one simulation step
    pub fn step(&mut self) {
        if self.vehicle.has_arrived {
            return;
        }

        // 1. CALCULATE FUZZY INPUTS
        let distance_to_target = euclidean_distance(
            &self.vehicle.state.position,
            &self.map.target.position,
        );

        // 2. CHECK ARRIVAL CONDITION FIRST (before moving)
        // Vehicle must satisfy BOTH distance and angle requirements to arrive
        let angle_error = (self.map.target.required_angle - self.vehicle.state.angle).abs();

        if distance_to_target < self.distance_threshold && angle_error < self.angle_threshold {
            self.vehicle.has_arrived = true;

            // Record final position before stopping
            self.trajectory.push(TrajectoryPoint {
                t: self.time,
                x: self.vehicle.state.position.x,
                y: self.vehicle.state.position.y,
                angle: self.vehicle.state.angle.to_degrees(),
                velocity: self.vehicle.state.velocity,
                distance_to_target,
            });

            sim_println!("\n✓ Vehicle arrived successfully at t={:.2}s", self.time);
            sim_println!("  Distance: {:.2} units, Angle error: {:.1}°", distance_to_target, angle_error.to_degrees());
            return;
        }

        // 3. CONTINUE NAVIGATION
        // Use interpolated angular error (navigates to target when far, aligns to 90° when close)
        let angular_error = compute_angular_error_with_arrival(
            &self.vehicle.state.position,
            self.vehicle.state.angle,
            &self.map.target,
            distance_to_target,
        );

        let velocity_relative = self.vehicle.state.velocity / self.vehicle.characteristics.max_velocity;

        // 4. EVALUATE FUZZY CONTROLLER
        let (angular_adjustment, _velocity_adjustment) =
            self.controller.compute_control(
                distance_to_target,
                angular_error,
                velocity_relative,
            );

        // 5. APPLY PHYSICAL CONSTRAINTS
        let angular_adjustment_clamped = clamp(
            angular_adjustment,
            -self.vehicle.characteristics.maneuverability,
            self.vehicle.characteristics.maneuverability,
        );

        // 6. UPDATE VEHICLE STATE
        // Update angle
        self.vehicle.state.angle += angular_adjustment_clamped * self.dt;
        self.vehicle.state.angle = normalize_angle(self.vehicle.state.angle);

        // Velocity remains constant (no velocity_adjustment applied)

        // 7. UPDATE POSITION (kinematic model)
        let old_position = self.vehicle.state.position.clone();
        let new_x = old_position.x + self.vehicle.state.velocity * self.vehicle.state.angle.cos() * self.dt;
        let new_y = old_position.y + self.vehicle.state.velocity * self.vehicle.state.angle.sin() * self.dt;

        self.vehicle.update_position(Point::new(new_x, new_y));

        // 8. UPDATE TIME
        self.time += self.dt;
        self.vehicle.time_elapsed = self.time;

        // 9. RECORD TRAJECTORY POINT
        self.trajectory.push(TrajectoryPoint {
            t: self.time,
            x: self.vehicle.state.position.x,
            y: self.vehicle.state.position.y,
            angle: self.vehicle.state.angle.to_degrees(),
            velocity: self.vehicle.state.velocity,
            distance_to_target,
        });
    }

    /// Run the complete simulation
    pub fn run(&mut self) -> SimulationResult {
        sim_println!("\n╔══════════════════════════════════════════════════════╗");
        sim_println!("║       FUZZY NAVIGATION SIMULATION STARTED           ║");
        sim_println!("╚══════════════════════════════════════════════════════╝\n");

        sim_println!("Vehicle Type: {} ", self.vehicle.vehicle_type.name());
        sim_println!("  - Size: {}", self.vehicle.characteristics.size);
        sim_println!("  - Max Speed: {:.1} units/s", self.vehicle.characteristics.max_velocity);
        sim_println!("  - Max Acceleration: {:.1} units/s²", self.vehicle.characteristics.max_acceleration);
        sim_println!("  - Maneuverability: {:.1}°/s\n", self.vehicle.characteristics.maneuverability.to_degrees());

        sim_println!("Map: {}x{}", self.map.width, self.map.height);
        sim_println!("Target: ({:.1}, {:.1})", self.map.target.position.x, self.map.target.position.y);
        sim_println!("Required arrival angle: {:.1}°\n", self.map.target.required_angle.to_degrees());

        sim_println!("Starting Position: ({:.1}, {:.1})",
            self.vehicle.state.position.x,
            self.vehicle.state.position.y);
        sim_println!("Starting Angle: {:.1}°\n", self.vehicle.state.angle.to_degrees());

        let _initial_distance = euclidean_distance(
            &self.vehicle.state.position,
            &self.map.target.position,
        );
        sim_println!("Initial Distance to Target: {:.1} units\n", _initial_distance);

        sim_println!("Running simulation (dt={:.3}s, max_time={:.1}s)...\n", self.dt, self.max_time);

        let mut step_count = 0;
        let print_interval = (5.0 / self.dt) as usize; // Print every 5 seconds

        while self.time < self.max_time && !self.vehicle.has_arrived {
            self.step();
            step_count += 1;

            if step_count % print_interval == 0 {
                let _dist = euclidean_distance(
                    &self.vehicle.state.position,
                    &self.map.target.position,
                );
                sim_println!(
                    "[t={:6.2}s] pos=({:6.1}, {:6.1}) vel={:5.1} dist={:6.1} angle={:6.1}°",
                    self.time,
                    self.vehicle.state.position.x,
                    self.vehicle.state.position.y,
                    self.vehicle.state.velocity,
                    _dist,
                    self.vehicle.state.angle.to_degrees()
                );
            }
        }

        let final_distance = euclidean_distance(
            &self.vehicle.state.position,
            &self.map.target.position,
        );
        let final_angle_error = (self.map.target.required_angle - self.vehicle.state.angle).abs();

        let metrics = SimulationMetrics {
            success: self.vehicle.has_arrived,
            arrival_time: if self.vehicle.has_arrived {
                Some(self.time)
            } else {
                None
            },
            distance_traveled: self.vehicle.distance_traveled,
            final_angle_error: final_angle_error.to_degrees(),
            final_distance_to_target: final_distance,
        };

        sim_println!("\n╔══════════════════════════════════════════════════════╗");
        sim_println!("║              SIMULATION COMPLETED                    ║");
        sim_println!("╚══════════════════════════════════════════════════════╝\n");

        sim_println!("Results:");
        sim_println!("  Success: {}", if metrics.success { "YES ✓" } else { "NO ✗" });
        if let Some(_t) = metrics.arrival_time {
            sim_println!("  Arrival Time: {:.2}s", _t);
        } else {
            sim_println!("  Status: Did not arrive (timeout at {:.2}s)", self.max_time);
        }
        sim_println!("  Distance Traveled: {:.2} units", metrics.distance_traveled);
        sim_println!("  Final Distance to Target: {:.2} units", metrics.final_distance_to_target);
        sim_println!("  Final Angle Error: {:.2}°", metrics.final_angle_error);
        sim_println!("  Total Steps: {}", step_count);

        SimulationResult {
            vehicle_type: self.vehicle.vehicle_type.name().to_string(),
            trajectory: self.trajectory.clone(),
            metrics,
        }
    }
}
