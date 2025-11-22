#!/bin/bash
# Script para ejecutar benchmark de navegacion difusa
#
# Uso:
#   ./benchmark.sh           # 30 iteraciones por defecto
#   ./benchmark.sh 100       # 100 iteraciones
#   ./benchmark.sh 50 100 200  # multiples configuraciones

set -e

echo "Compilando en modo release..."
cargo build --release --bin benchmark

if [ $# -eq 0 ]; then
    echo ""
    echo "Ejecutando benchmark con 30 iteraciones..."
    ./target/release/benchmark 30
else
    for n in "$@"; do
        echo ""
        echo "========================================"
        echo "Ejecutando benchmark con $n iteraciones"
        echo "========================================"
        ./target/release/benchmark "$n"
    done
fi

echo ""
echo "Archivos generados en output/:"
ls -la output/benchmark_*.{json,csv} 2>/dev/null || true
