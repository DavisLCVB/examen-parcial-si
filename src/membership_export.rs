// Module for exporting membership function visualizations

use crate::fuzzy_system::LinguisticVariable;
use crate::vehicle::{create_vehicle_preset, VehicleType};
use plotters::prelude::*;
use std::fs;

const IMAGE_WIDTH: u32 = 800;
const IMAGE_HEIGHT: u32 = 600;

/// Export all membership functions for a given linguistic variable
pub fn export_variable_memberships(
    variable: &LinguisticVariable,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (IMAGE_WIDTH, IMAGE_HEIGHT))
        .into_drawing_area();
    root.fill(&WHITE)?;

    let (min, max) = variable.range;
    let name = &variable.name;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Funciones de Pertenencia: {}", name), ("sans-serif", 40))
        .margin(15)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(min..max, 0.0..1.1)?;

    chart
        .configure_mesh()
        .x_desc("Valor")
        .y_desc("Grado de Pertenencia")
        .draw()?;

    // Color palette for different sets
    let colors = vec![
        &RED,
        &BLUE,
        &GREEN,
        &MAGENTA,
        &CYAN,
        &RGBColor(255, 165, 0), // Orange
        &RGBColor(128, 0, 128), // Purple
        &RGBColor(255, 192, 203), // Pink
    ];

    // Plot each fuzzy set
    for (idx, set) in variable.fuzzy_sets.iter().enumerate() {
        let color = colors[idx % colors.len()];
        let num_points = 200;
        let step = (max - min) / num_points as f64;

        let points: Vec<(f64, f64)> = (0..=num_points)
            .map(|i| {
                let x = min + i as f64 * step;
                let y = set.membership_function.evaluate(x);
                (x, y)
            })
            .collect();

        chart
            .draw_series(LineSeries::new(points, color.stroke_width(2)))?
            .label(&set.name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(3)));
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}

/// Export all membership functions from the navigation controller
pub fn export_navigation_memberships(
    vehicle_type: VehicleType,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Export all navigation variables for this vehicle type
    export_all_navigation_variables(vehicle_type, output_dir)?;

    Ok(())
}

/// Export all navigation system variables for all vehicle types
pub fn export_all_navigation_variables(
    vehicle_type: VehicleType,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::fuzzy_system::{triangular, trapezoidal, FuzzySet, LinguisticVariable};
    use std::f64::consts::PI;

    let characteristics = create_vehicle_preset(vehicle_type);
    let maneuverability = characteristics.maneuverability;

    let vehicle_dir = format!("{}/{}", output_dir, vehicle_type.name());
    fs::create_dir_all(&vehicle_dir)?;

    println!("\nExportando funciones de pertenencia para {}...", vehicle_type.name());

    // INPUT 1: distancia_al_objetivo
    let mut dist_var = LinguisticVariable::new("distancia_al_objetivo", (0.0, 1000.0));
    dist_var.add_set(FuzzySet::new("muy_cerca", trapezoidal(0.0, 0.0, 50.0, 100.0)));
    dist_var.add_set(FuzzySet::new("media", triangular(80.0, 200.0, 400.0)));
    dist_var.add_set(FuzzySet::new("lejos", trapezoidal(350.0, 500.0, 1000.0, 1000.0)));

    let path = format!("{}/input_distancia_al_objetivo.png", vehicle_dir);
    export_variable_memberships(&dist_var, &path)?;
    println!("  ✓ {}", path);

    // INPUT 2: error_angular
    let mut error_var = LinguisticVariable::new("error_angular", (-PI, PI));
    error_var.add_set(FuzzySet::new(
        "alineado",
        trapezoidal(-10f64.to_radians(), -5f64.to_radians(), 5f64.to_radians(), 10f64.to_radians()),
    ));
    error_var.add_set(FuzzySet::new(
        "desviado_izq",
        triangular(-90f64.to_radians(), -45f64.to_radians(), -10f64.to_radians()),
    ));
    error_var.add_set(FuzzySet::new(
        "desviado_der",
        triangular(10f64.to_radians(), 45f64.to_radians(), 90f64.to_radians()),
    ));
    error_var.add_set(FuzzySet::new(
        "muy_desviado_izq",
        trapezoidal(-PI, -150f64.to_radians(), -120f64.to_radians(), -70f64.to_radians()),
    ));
    error_var.add_set(FuzzySet::new(
        "muy_desviado_der",
        trapezoidal(70f64.to_radians(), 120f64.to_radians(), 150f64.to_radians(), PI),
    ));

    let path = format!("{}/input_error_angular.png", vehicle_dir);
    export_variable_memberships(&error_var, &path)?;
    println!("  ✓ {}", path);

    // INPUT 3: velocidad_relativa
    let mut vel_var = LinguisticVariable::new("velocidad_relativa", (0.0, 1.0));
    vel_var.add_set(FuzzySet::new("lenta", triangular(0.0, 0.0, 0.3)));
    vel_var.add_set(FuzzySet::new("media", triangular(0.2, 0.5, 0.8)));
    vel_var.add_set(FuzzySet::new("rapida", trapezoidal(0.7, 1.0, 1.0, 1.0)));

    let path = format!("{}/input_velocidad_relativa.png", vehicle_dir);
    export_variable_memberships(&vel_var, &path)?;
    println!("  ✓ {}", path);

    // OUTPUT 1: ajuste_angular
    let mut ang_out_var = LinguisticVariable::new("ajuste_angular", (-maneuverability, maneuverability));
    ang_out_var.add_set(FuzzySet::new(
        "girar_izq",
        triangular(-maneuverability, -0.7 * maneuverability, -0.3 * maneuverability),
    ));
    ang_out_var.add_set(FuzzySet::new(
        "leve_izq",
        triangular(-0.4 * maneuverability, -0.2 * maneuverability, 0.0),
    ));
    ang_out_var.add_set(FuzzySet::new(
        "mantener",
        triangular(-0.1 * maneuverability, 0.0, 0.1 * maneuverability),
    ));
    ang_out_var.add_set(FuzzySet::new(
        "leve_der",
        triangular(0.0, 0.2 * maneuverability, 0.4 * maneuverability),
    ));
    ang_out_var.add_set(FuzzySet::new(
        "girar_der",
        triangular(0.3 * maneuverability, 0.7 * maneuverability, maneuverability),
    ));

    let path = format!("{}/output_ajuste_angular.png", vehicle_dir);
    export_variable_memberships(&ang_out_var, &path)?;
    println!("  ✓ {}", path);

    Ok(())
}

/// Export membership functions for all vehicle types
pub fn export_all_vehicle_types(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════╗");
    println!("║   EXPORTANDO FUNCIONES DE PERTENENCIA                ║");
    println!("╚══════════════════════════════════════════════════════╝");

    let vehicle_types = vec![
        VehicleType::Heavy,
        VehicleType::Standard,
        VehicleType::Agile,
    ];

    for vehicle_type in vehicle_types {
        export_all_navigation_variables(vehicle_type, output_dir)?;
    }

    println!("\n✓ Todas las funciones de pertenencia exportadas a: {}/", output_dir);
    Ok(())
}
