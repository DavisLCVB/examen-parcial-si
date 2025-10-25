#!/bin/bash
# Helper script to run simulation and visualizer

echo "╔══════════════════════════════════════════════════════╗"
echo "║  Fuzzy Vehicle Navigation - Run Script              ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""

# Check if output directory exists
mkdir -p output

# Step 1: Run simulation
echo "Step 1: Running navigation simulation..."
echo "----------------------------------------"
cargo run --bin navigation

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ Simulation failed!"
    exit 1
fi

echo ""
echo "✅ Simulation completed successfully!"
echo ""

# Check if JSON was generated
if [ ! -f "output/trajectory_standard.json" ]; then
    echo "❌ Trajectory file not found!"
    exit 1
fi

echo "Step 2: Launching visualizer..."
echo "----------------------------------------"
echo "Controls:"
echo "  SPACE - Play/Pause"
echo "  ← →   - Change playback speed"
echo "  R     - Restart animation"
echo ""
sleep 2

# Step 2: Run visualizer
cargo run --bin visualizer
