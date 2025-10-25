// Vehicle module - Vehicle structures, types and configuration presets

use crate::map::Point;
use serde::{Serialize, Deserialize};

/// Physical and performance characteristics of a vehicle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleCharacteristics {
    pub size: f64,                    // Radius or characteristic dimension
    pub maneuverability: f64,         // Maximum turning rate (degrees/second)
    pub max_velocity: f64,            // Maximum speed (units/second)
    pub max_acceleration: f64,        // Maximum acceleration (units/second²)
}

/// Dynamic state of a vehicle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleState {
    pub position: Point,
    pub angle: f64,                   // Orientation in radians (0 = east, π/2 = north)
    pub velocity: f64,                // Current speed (units/second)
}

/// Vehicle types with predefined characteristics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VehicleType {
    Heavy,          // Tipo A: Vehículo Pesado
    Standard,       // Tipo B: Vehículo Estándar
    Agile,          // Tipo C: Vehículo Ágil
    UltraAgile,     // Tipo D: Vehículo Ultra-Maniobrable
}

impl VehicleType {
    pub fn name(&self) -> &str {
        match self {
            VehicleType::Heavy => "Barco",
            VehicleType::Standard => "Lancha",
            VehicleType::Agile => "Avión",
            VehicleType::UltraAgile => "Ultra-Agile",
        }
    }
}

/// Complete vehicle structure
#[derive(Debug, Clone)]
pub struct Vehicle {
    pub vehicle_type: VehicleType,
    pub characteristics: VehicleCharacteristics,
    pub state: VehicleState,

    // Mission tracking
    pub has_arrived: bool,
    pub distance_traveled: f64,
    pub time_elapsed: f64,
}

impl Vehicle {
    /// Create a new vehicle with specified type and initial state
    pub fn new(
        vehicle_type: VehicleType,
        characteristics: VehicleCharacteristics,
        initial_position: Point,
        initial_angle: f64,
    ) -> Self {
        Self {
            vehicle_type,
            characteristics,
            state: VehicleState {
                position: initial_position,
                angle: initial_angle,
                velocity: 0.0,
            },
            has_arrived: false,
            distance_traveled: 0.0,
            time_elapsed: 0.0,
        }
    }

    /// Update vehicle position and track distance
    pub fn update_position(&mut self, new_position: Point) {
        let dx = new_position.x - self.state.position.x;
        let dy = new_position.y - self.state.position.y;
        let distance_step = (dx * dx + dy * dy).sqrt();

        self.distance_traveled += distance_step;
        self.state.position = new_position;
    }
}

/// Factory function to create vehicle presets from the specification
pub fn create_vehicle_preset(vehicle_type: VehicleType) -> VehicleCharacteristics {
    match vehicle_type {
        VehicleType::Heavy => VehicleCharacteristics {
            size: 15.0,
            maneuverability: 20.0f64.to_radians(),  // Convert degrees to radians/second
            max_velocity: 50.0,
            max_acceleration: 10.0,
        },
        VehicleType::Standard => VehicleCharacteristics {
            size: 10.0,
            maneuverability: 35.0f64.to_radians(),
            max_velocity: 80.0,
            max_acceleration: 20.0,
        },
        VehicleType::Agile => VehicleCharacteristics {
            size: 6.0,
            maneuverability: 60.0f64.to_radians(),
            max_velocity: 100.0,
            max_acceleration: 30.0,
        },
        VehicleType::UltraAgile => VehicleCharacteristics {
            size: 8.0,
            maneuverability: 90.0f64.to_radians(),
            max_velocity: 70.0,
            max_acceleration: 25.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_presets() {
        let heavy = create_vehicle_preset(VehicleType::Heavy);
        assert_eq!(heavy.size, 15.0);
        assert_eq!(heavy.max_velocity, 50.0);

        let agile = create_vehicle_preset(VehicleType::Agile);
        assert_eq!(agile.size, 6.0);
        assert_eq!(agile.max_velocity, 100.0);
    }

    #[test]
    fn test_vehicle_creation() {
        let characteristics = create_vehicle_preset(VehicleType::Standard);
        let vehicle = Vehicle::new(
            VehicleType::Standard,
            characteristics,
            Point::new(100.0, 50.0),
            90f64.to_radians(),
        );

        assert_eq!(vehicle.state.velocity, 0.0);
        assert!(!vehicle.has_arrived);
        assert_eq!(vehicle.distance_traveled, 0.0);
    }
}
