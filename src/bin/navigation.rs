// Multi-Vehicle Navigation Simulation - Fuzzy logic-based navigation for 3 vehicles
//
// Run with: cargo run --bin navigation

use examen_parcial::map::Map;
use examen_parcial::simulation::{Simulation, MultiVehicleSimulationResult, VehicleResult};
use examen_parcial::vehicle::VehicleType;
use std::fs;
use std::io::Write;

fn main() {
    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║   MULTI-VEHICLE FUZZY NAVIGATION SIMULATION          ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Create map (1000x800, target at top center: 500,700)
    let map = Map::new(1000.0, 800.0, 500.0, 700.0);

    let dt = 0.05; // 50ms time step
    let max_time = 600.0;

    // Create 3 vehicles of different types
    let vehicle_types = vec![
        VehicleType::Heavy,
        VehicleType::Standard,
        VehicleType::Agile,
    ];

    let mut simulations: Vec<Simulation> = vehicle_types.iter()
        .map(|&vtype| Simulation::new(map.clone(), vtype, dt, max_time))
        .collect();

    println!("Simulating {} vehicles:", simulations.len());
    for (i, sim) in simulations.iter().enumerate() {
        println!("  {}. {} - Start: ({:.1}, {:.1}) @ {:.1}°",
            i + 1,
            sim.vehicle.vehicle_type.name(),
            sim.vehicle.state.position.x,
            sim.vehicle.state.position.y,
            sim.vehicle.state.angle.to_degrees()
        );
    }
    println!("\nTarget: (500.0, 700.0) @ 90°\n");
    println!("Running simulation (dt={:.3}s, max_time={:.1}s)...\n", dt, max_time);

    // Run all simulations in parallel
    let mut time = 0.0;
    let mut all_arrived = false;
    let mut step_count = 0;

    while time < max_time && !all_arrived {
        // Update each vehicle
        for sim in &mut simulations {
            if !sim.vehicle.has_arrived {
                sim.step();
            }
        }

        time += dt;
        step_count += 1;

        // Check if all have arrived
        all_arrived = simulations.iter().all(|s| s.vehicle.has_arrived);

        // Print progress every 5 seconds
        if step_count % 100 == 0 {
            let arrived_count = simulations.iter().filter(|s| s.vehicle.has_arrived).count();
            println!("[t={:6.2}s] {}/{} vehicles arrived", time, arrived_count, simulations.len());
        }
    }

    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║            SIMULATION COMPLETED                       ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Collect results
    let mut vehicle_results = Vec::new();

    for (i, sim) in simulations.into_iter().enumerate() {
        println!("Vehicle {}: {}", i + 1, sim.vehicle.vehicle_type.name());

        // Calculate metrics
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

        let metrics = examen_parcial::simulation::SimulationMetrics {
            success,
            arrival_time,
            distance_traveled,
            final_angle_error,
            final_distance_to_target: final_distance,
        };

        println!("  Success: {}", if success { "YES ✓" } else { "NO ✗" });
        if let Some(t) = arrival_time {
            println!("  Arrival Time: {:.2}s", t);
        }
        println!("  Distance Traveled: {:.2} units", distance_traveled);
        println!("  Final Distance: {:.2} units", final_distance);
        println!("  Final Angle Error: {:.2}°", final_angle_error);
        println!();

        vehicle_results.push(VehicleResult {
            vehicle_type: sim.vehicle.vehicle_type.name().to_string(),
            trajectory: sim.trajectory.clone(),
            metrics,
        });
    }

    // Create multi-vehicle result
    let multi_result = MultiVehicleSimulationResult {
        vehicles: vehicle_results,
        total_simulation_time: time,
    };

    // Export to JSON
    let json_output = serde_json::to_string_pretty(&multi_result)
        .expect("Failed to serialize simulation result");

    fs::create_dir_all("output").expect("Failed to create output directory");

    let filename = "output/trajectory_multi.json";
    let mut file = fs::File::create(filename).expect("Failed to create output file");
    file.write_all(json_output.as_bytes())
        .expect("Failed to write to file");

    println!("✓ Multi-vehicle trajectory exported to: {}", filename);
    println!("\nVisualize with: cargo run --bin visualizer");
}
