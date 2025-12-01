#!/bin/bash
# Script para probar la API localmente

BASE_URL="${BASE_URL:-http://localhost:8000}"

echo "🧪 Testing Fuzzy Navigation System API"
echo "Base URL: $BASE_URL"
echo ""

# Test 1: Health Check
echo "1️⃣  Testing Health Check..."
curl -s "$BASE_URL/health" | jq '.'
echo ""
echo ""

# Test 2: Simple Simulation
echo "2️⃣  Testing Simple Simulation (default parameters)..."
curl -s -X POST "$BASE_URL/api/simulate" \
  -H "Content-Type: application/json" \
  -d '{}' | jq '.message, .total_simulation_time, .vehicles[].metrics.success'
echo ""
echo ""

# Test 3: Single Vehicle Simulation
echo "3️⃣  Testing Single Vehicle Simulation (Standard)..."
curl -s -X POST "$BASE_URL/api/simulate" \
  -H "Content-Type: application/json" \
  -d '{"vehicle_types": ["Standard"], "max_time": 300}' | jq '.message'
echo ""
echo ""

# Test 4: Quick Benchmark
echo "4️⃣  Testing Quick Benchmark (5 iterations)..."
curl -s -X POST "$BASE_URL/api/benchmark" \
  -H "Content-Type: application/json" \
  -d '{"iterations": 5, "vehicle_types": ["Standard"]}' | jq '.message, .aggregate_stats[].success_rate'
echo ""
echo ""

echo "✅ All tests completed!"
