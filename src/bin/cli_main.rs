// Unified entry point for the Fuzzy Navigation System
//
// Run with:
//   cargo run -- --mode navigation [--iterations N]
//   cargo run -- --mode benchmark [--iterations N]
//   cargo run -- --mode visualizer
//   cargo run -- --mode export-memberships [--output-dir DIR]

use clap::Parser;
use examen_parcial::membership_export;
use std::process;

mod navigation_runner;
mod benchmark_runner;
mod visualizer_runner;

#[derive(Parser, Debug)]
#[command(author, version, about = "Sistema de Navegación Difusa Multi-Vehículo", long_about = None)]
struct Args {
    #[arg(short, long, value_name = "MODE")]
    #[arg(help = "Modo de ejecución: navigation, benchmark, visualizer, export-memberships")]
    mode: String,

    #[arg(short, long, default_value_t = 30)]
    #[arg(help = "Número de iteraciones (solo para benchmark)")]
    iterations: usize,

    #[arg(short, long, default_value = "output/memberships")]
    #[arg(help = "Directorio de salida para exportar funciones de pertenencia")]
    output_dir: String,

    #[arg(short = 't', long)]
    #[arg(help = "Número de threads para benchmark paralelo (por defecto: mitad de los cores disponibles)")]
    threads: Option<usize>,
}

fn main() {
    let args = Args::parse();

    match args.mode.to_lowercase().as_str() {
        "navigation" | "nav" => {
            println!("\n╔══════════════════════════════════════════════════════╗");
            println!("║   MODO: NAVEGACIÓN MULTI-VEHÍCULO                    ║");
            println!("╚══════════════════════════════════════════════════════╝\n");
            navigation_runner::run();
        }

        "benchmark" | "bench" => {
            println!("\n╔══════════════════════════════════════════════════════╗");
            println!("║   MODO: BENCHMARK                                    ║");
            println!("╚══════════════════════════════════════════════════════╝\n");
            benchmark_runner::run(args.iterations, args.threads);
        }

        "visualizer" | "viz" | "visual" => {
            println!("\n╔══════════════════════════════════════════════════════╗");
            println!("║   MODO: VISUALIZADOR                                 ║");
            println!("╚══════════════════════════════════════════════════════╝\n");
            visualizer_runner::run();
        }

        "export-memberships" | "export" => {
            println!("\n╔══════════════════════════════════════════════════════╗");
            println!("║   MODO: EXPORTAR FUNCIONES DE PERTENENCIA           ║");
            println!("╚══════════════════════════════════════════════════════╝\n");

            if let Err(e) = membership_export::export_all_vehicle_types(&args.output_dir) {
                eprintln!("\nError al exportar funciones de pertenencia: {}", e);
                process::exit(1);
            }

            println!("\n✓ Exportación completada exitosamente!");
        }

        _ => {
            eprintln!("\n❌ Error: Modo desconocido '{}'\n", args.mode);
            eprintln!("Modos válidos:");
            eprintln!("  - navigation (nav)         : Ejecutar simulación de navegación");
            eprintln!("  - benchmark (bench)        : Ejecutar múltiples simulaciones para estadísticas");
            eprintln!("  - visualizer (viz, visual) : Abrir el visualizador interactivo");
            eprintln!("  - export-memberships (export) : Exportar gráficos de funciones de pertenencia");
            eprintln!("\nEjemplos:");
            eprintln!("  cargo run -- --mode navigation");
            eprintln!("  cargo run -- --mode benchmark --iterations 100");
            eprintln!("  cargo run -- --mode benchmark --iterations 100 --threads 4  # Limitar a 4 threads");
            eprintln!("  cargo run -- --mode visualizer");
            eprintln!("  cargo run -- --mode export-memberships --output-dir output/plots\n");
            process::exit(1);
        }
    }
}
