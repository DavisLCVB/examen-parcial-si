# Fuzzy Navigation System API

API REST para el Sistema de Navegación Difusa Multi-Vehículo, desplegable en Shuttle.

## Despliegue

### Desarrollo Local

```bash
# Instalar Shuttle CLI
cargo install cargo-shuttle

# Ejecutar localmente
cargo shuttle run

# La API estará disponible en http://localhost:8000
```

### Despliegue en Shuttle

```bash
# Login a Shuttle
cargo shuttle login

# Desplegar
cargo shuttle deploy

# Ver logs
cargo shuttle logs
```

## Endpoints

### Health Check

**GET** `/` o `/health`

Verifica el estado de la API.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "message": "Fuzzy Navigation System API is running"
}
```

---

### Ejecutar Simulación

**POST** `/api/simulate`

Ejecuta una simulación de navegación con uno o más vehículos.

**Request Body:**
```json
{
  "vehicle_types": ["Heavy", "Standard", "Agile"],
  "dt": 0.05,
  "max_time": 600.0,
  "map_width": 1000.0,
  "map_height": 800.0,
  "target_x": 500.0,
  "target_y": 700.0
}
```

**Parámetros:**
- `vehicle_types` (opcional): Array de tipos de vehículos. Default: `["Heavy", "Standard", "Agile"]`
- `dt` (opcional): Paso de tiempo en segundos. Default: `0.05`
- `max_time` (opcional): Tiempo máximo de simulación. Default: `600.0`
- `map_width` (opcional): Ancho del mapa. Default: `1000.0`
- `map_height` (opcional): Alto del mapa. Default: `800.0`
- `target_x` (opcional): Coordenada X del objetivo. Default: `500.0`
- `target_y` (opcional): Coordenada Y del objetivo. Default: `700.0`

**Response:**
```json
{
  "success": true,
  "vehicles": [
    {
      "vehicle_type": "Heavy",
      "trajectory": [
        {
          "t": 0.0,
          "x": 100.0,
          "y": 200.0,
          "angle": 45.0,
          "velocity": 10.0,
          "distance_to_target": 500.0
        }
      ],
      "metrics": {
        "success": true,
        "arrival_time": 125.5,
        "distance_traveled": 750.2,
        "final_angle_error": 1.5,
        "final_distance_to_target": 15.0
      }
    }
  ],
  "total_simulation_time": 150.0,
  "message": "Simulation completed: 3/3 vehicles arrived successfully"
}
```

**Ejemplo con curl:**
```bash
curl -X POST http://localhost:8000/api/simulate \
  -H "Content-Type: application/json" \
  -d '{
    "vehicle_types": ["Standard"],
    "max_time": 300.0
  }'
```

**Ejemplo con JavaScript:**
```javascript
const response = await fetch('http://localhost:8000/api/simulate', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    vehicle_types: ['Heavy', 'Agile'],
    max_time: 400.0
  })
});

const result = await response.json();
console.log(result);
```

---

### Ejecutar Benchmark

**POST** `/api/benchmark`

Ejecuta múltiples iteraciones de simulaciones para obtener estadísticas agregadas.

**Request Body:**
```json
{
  "iterations": 30,
  "vehicle_types": ["Heavy", "Standard", "Agile"],
  "threads": 4,
  "dt": 0.05,
  "max_time": 600.0
}
```

**Parámetros:**
- `iterations` (opcional): Número de iteraciones. Default: `30`
- `vehicle_types` (opcional): Array de tipos de vehículos. Default: `["Heavy", "Standard", "Agile"]`
- `threads` (opcional): Número de threads para procesamiento paralelo. Default: mitad de cores disponibles
- `dt` (opcional): Paso de tiempo en segundos. Default: `0.05`
- `max_time` (opcional): Tiempo máximo de simulación. Default: `600.0`

**Response:**
```json
{
  "success": true,
  "num_iterations": 30,
  "aggregate_stats": [
    {
      "vehicle_type": "Heavy",
      "total_runs": 30,
      "successes": 28,
      "success_rate": 93.33,
      "avg_arrival_time": 145.2,
      "std_arrival_time": 12.5,
      "min_arrival_time": 125.0,
      "max_arrival_time": 165.0,
      "avg_distance_traveled": 780.5,
      "std_distance_traveled": 45.2,
      "avg_final_distance": 18.3,
      "avg_final_angle_error": 1.2
    }
  ],
  "message": "Benchmark completed: 30 iterations across 3 vehicle types"
}
```

**Ejemplo con curl:**
```bash
curl -X POST http://localhost:8000/api/benchmark \
  -H "Content-Type: application/json" \
  -d '{
    "iterations": 50,
    "vehicle_types": ["Standard"],
    "threads": 8
  }'
```

---

## Tipos de Vehículos

La API soporta tres tipos de vehículos:

- **Heavy**: Vehículo pesado con baja maniobrabilidad
- **Standard**: Vehículo estándar con características balanceadas
- **Agile**: Vehículo ágil con alta maniobrabilidad

## Códigos de Error

- `400 Bad Request`: Parámetros inválidos o tipos de vehículos desconocidos
- `500 Internal Server Error`: Error durante la ejecución de la simulación

**Formato de error:**
```json
{
  "error": "400 Bad Request",
  "details": "Unknown vehicle type: SuperFast. Valid types: Heavy, Standard, Agile"
}
```

## Características

- ✅ Simulaciones de navegación difusa
- ✅ Soporte para múltiples tipos de vehículos
- ✅ Benchmarking paralelo con Rayon
- ✅ CORS habilitado
- ✅ Trazabilidad con tracing
- ✅ Respuestas JSON estructuradas
- ✅ Validación de parámetros

## Arquitectura

```
┌─────────────┐
│   Cliente   │
└──────┬──────┘
       │ HTTP/JSON
       ▼
┌─────────────┐
│  Axum API   │
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│  Handlers (async)   │
└──────┬──────────────┘
       │ spawn_blocking
       ▼
┌─────────────────────┐
│ Simulation Logic    │
│  (blocking/sync)    │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│  Fuzzy Controller   │
│  + Physics Engine   │
└─────────────────────┘
```

## Limitaciones en Shuttle

- No se incluyen las funcionalidades de visualización gráfica (macroquad)
- Los resultados no se guardan en archivos (solo se retornan vía JSON)
- Para usar la CLI original, compilar con: `cargo build --features cli --bin cli`

## Desarrollo

### Ejecutar CLI (modo original)

```bash
cargo run --features cli --bin cli -- --mode navigation
cargo run --features cli --bin cli -- --mode benchmark --iterations 100
```

### Ejecutar API (modo web)

```bash
cargo shuttle run
# o para desarrollo sin Shuttle:
cargo run
```

## Testing

```bash
# Test básico del health endpoint
curl http://localhost:8000/health

# Test de simulación simple
curl -X POST http://localhost:8000/api/simulate \
  -H "Content-Type: application/json" \
  -d '{}'

# Test de benchmark rápido
curl -X POST http://localhost:8000/api/benchmark \
  -H "Content-Type: application/json" \
  -d '{"iterations": 5}'
```

## Licencia

Ver LICENSE file.
