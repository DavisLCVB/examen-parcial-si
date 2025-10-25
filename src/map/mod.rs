// Map module - Environment configuration for vehicle navigation

use std::f64::consts::PI;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct StartZone {
    pub height_percentage: f64,  // Percentage of map height (e.g., 0.08 for 8%)
}

#[derive(Debug, Clone)]
pub struct Target {
    pub position: Point,
    pub required_angle: f64,  // Required arrival angle in radians (π/2 for 90°)
}

#[derive(Debug, Clone)]
pub struct Map {
    pub width: f64,
    pub height: f64,
    pub start_zone: StartZone,
    pub target: Target,
}

impl Map {
    pub fn new(width: f64, height: f64, target_x: f64, target_y: f64) -> Self {
        Self {
            width,
            height,
            start_zone: StartZone {
                height_percentage: 0.08,  // 8% of map height
            },
            target: Target {
                position: Point::new(target_x, target_y),
                required_angle: PI / 2.0,  // 90 degrees
            },
        }
    }

    /// Generate a random starting position within the start zone
    pub fn random_start_position(&self) -> Point {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let x = rng.gen_range(0.0..self.width);
        let y = rng.gen_range(0.0..(self.height * self.start_zone.height_percentage));

        Point::new(x, y)
    }

    /// Generate a random initial angle (generally pointing upward)
    pub fn random_start_angle(&self) -> f64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Random angle between 30° and 150° (biased upward)
        rng.gen_range(30f64.to_radians()..150f64.to_radians())
    }

    /// Generate a random initial velocity percentage (5% to 15% of max velocity)
    pub fn random_start_velocity_percentage(&self) -> f64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Random percentage between 5% and 15%
        rng.gen_range(0.05..0.15)
    }
}

// Geometry utility functions

/// Calculate Euclidean distance between two points
pub fn euclidean_distance(p1: &Point, p2: &Point) -> f64 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    (dx * dx + dy * dy).sqrt()
}

/// Normalize angle to range [-π, π]
pub fn normalize_angle(angle: f64) -> f64 {
    let mut normalized = angle;
    while normalized > PI {
        normalized -= 2.0 * PI;
    }
    while normalized < -PI {
        normalized += 2.0 * PI;
    }
    normalized
}

/// Calculate angular error between current orientation and target direction
/// Returns angle in radians [-π, π]
pub fn compute_angular_error(current_pos: &Point, current_angle: f64, target_pos: &Point) -> f64 {
    let dx = target_pos.x - current_pos.x;
    let dy = target_pos.y - current_pos.y;

    let desired_angle = dy.atan2(dx);
    normalize_angle(desired_angle - current_angle)
}

/// Calculate angular error with arrival angle consideration
/// Uses a virtual approach point that converges to target as vehicle gets closer
///
/// Strategy for high-precision 90° arrival (±2°):
/// - When far (>120 units): Navigates directly to target
/// - When close (<120 units): Navigates to dynamic approach point below target
/// - Offset decreases with cubic curve for smoother final approach
///
/// Returns angle in radians [-π, π]
pub fn compute_angular_error_with_arrival(
    current_pos: &Point,
    current_angle: f64,
    target: &Target,
    distance_to_target: f64,
) -> f64 {
    const APPROACH_START: f64 = 120.0;    // When to start using approach point (increased for smoother approach)
    const MAX_OFFSET: f64 = 100.0;         // Maximum offset at APPROACH_START distance

    if distance_to_target > APPROACH_START {
        // Far away: navigate directly to target
        compute_angular_error(current_pos, current_angle, &target.position)
    } else {
        // Close: navigate to dynamic approach point that converges to target
        // Use cubic curve for smoother final approach: offset = MAX_OFFSET * (distance/START)^1.5
        let t = distance_to_target / APPROACH_START;
        let offset = MAX_OFFSET * t.powf(1.5);  // Cubic-like curve: approaches faster, then slows

        let approach_point = Point::new(
            target.position.x,
            target.position.y - offset  // Point below target (lower Y), vehicle approaches upward to arrive at 90°
        );

        compute_angular_error(current_pos, current_angle, &approach_point)
    }
}

/// Clamp a value between min and max
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert!((euclidean_distance(&p1, &p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_angle() {
        assert!((normalize_angle(7.0) - (7.0 - 2.0 * PI)).abs() < 0.001);
        assert!((normalize_angle(-7.0) - (-7.0 + 2.0 * PI)).abs() < 0.001);
        assert!((normalize_angle(PI) - PI).abs() < 0.001);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }
}
