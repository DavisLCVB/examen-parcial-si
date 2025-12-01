# Especificación de API para Frontend - Sistema de Navegación Difusa

Genera un frontend en Next.js 14+ (App Router) con TypeScript para consumir esta API REST.

## URL Base de la API

- **Desarrollo Local**: `http://localhost:8010`
- **Producción**: (URL de Shuttle después del deploy)

## Endpoints

### 1. Health Check

**Endpoint**: `GET /health`

**Descripción**: Verifica que la API esté funcionando correctamente.

**Request**: No requiere body ni parámetros.

**Response Success** (200):
```typescript
interface HealthResponse {
  status: string;        // "healthy"
  version: string;       // "0.1.0"
  message: string;       // "Fuzzy Navigation System API is running"
}
```

**Ejemplo de Response**:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "message": "Fuzzy Navigation System API is running"
}
```

---

### 2. Ejecutar Simulación

**Endpoint**: `POST /api/simulate`

**Descripción**: Ejecuta una simulación de navegación con uno o más vehículos y retorna las trayectorias completas y métricas de desempeño.

**Request Body** (todos los campos son opcionales):
```typescript
interface SimulationRequest {
  vehicle_types?: string[];  // Default: ["Heavy", "Standard", "Agile"]
  dt?: number;               // Time step in seconds. Default: 0.05
  max_time?: number;         // Max simulation time. Default: 600.0
  map_width?: number;        // Map width. Default: 1000.0
  map_height?: number;       // Map height. Default: 800.0
  target_x?: number;         // Target X coordinate. Default: 500.0
  target_y?: number;         // Target Y coordinate. Default: 700.0
}
```

**Valores válidos para `vehicle_types`**:
- `"Heavy"` - Vehículo pesado con baja maniobrabilidad
- `"Standard"` - Vehículo estándar balanceado
- `"Agile"` - Vehículo ágil con alta maniobrabilidad

**Response Success** (200):
```typescript
interface SimulationResponse {
  success: boolean;
  vehicles: VehicleSimulationResult[];
  total_simulation_time: number;
  message: string;
}

interface VehicleSimulationResult {
  vehicle_type: string;
  trajectory: TrajectoryPoint[];
  metrics: SimulationMetrics;
}

interface TrajectoryPoint {
  t: number;                    // Time in seconds
  x: number;                    // X position
  y: number;                    // Y position
  angle: number;                // Angle in degrees
  velocity: number;             // Current velocity
  distance_to_target: number;   // Distance to target
}

interface SimulationMetrics {
  success: boolean;                 // Did vehicle arrive?
  arrival_time: number | null;      // Time to arrival (null if didn't arrive)
  distance_traveled: number;        // Total distance traveled
  final_angle_error: number;        // Final angle error in degrees
  final_distance_to_target: number; // Final distance to target
}
```

**Ejemplo de Request**:
```json
{
  "vehicle_types": ["Standard", "Agile"],
  "max_time": 300.0
}
```

**Ejemplo de Response**:
```json
{
  "success": true,
  "vehicles": [
    {
      "vehicle_type": "Standard",
      "trajectory": [
        {
          "t": 0.0,
          "x": 150.5,
          "y": 200.3,
          "angle": 45.2,
          "velocity": 10.5,
          "distance_to_target": 580.2
        }
        // ... más puntos de trayectoria
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
  "message": "Simulation completed: 2/2 vehicles arrived successfully"
}
```

**Response Error** (400):
```typescript
interface ErrorResponse {
  error: string;          // HTTP status code text
  details: string | null; // Error description
}
```

---

### 3. Ejecutar Benchmark

**Endpoint**: `POST /api/benchmark`

**Descripción**: Ejecuta múltiples simulaciones (iteraciones) para obtener estadísticas agregadas de desempeño de los vehículos.

**Request Body** (todos los campos son opcionales):
```typescript
interface BenchmarkRequest {
  iterations?: number;       // Number of iterations. Default: 30
  vehicle_types?: string[];  // Default: ["Heavy", "Standard", "Agile"]
  threads?: number;          // Number of threads. Default: half of available cores
  dt?: number;               // Time step. Default: 0.05
  max_time?: number;         // Max simulation time. Default: 600.0
}
```

**Response Success** (200):
```typescript
interface BenchmarkResponse {
  success: boolean;
  num_iterations: number;
  aggregate_stats: AggregateStats[];
  message: string;
}

interface AggregateStats {
  vehicle_type: string;
  total_runs: number;
  successes: number;
  success_rate: number;            // Percentage (0-100)
  avg_arrival_time: number;
  std_arrival_time: number;        // Standard deviation
  min_arrival_time: number;
  max_arrival_time: number;
  avg_distance_traveled: number;
  std_distance_traveled: number;   // Standard deviation
  avg_final_distance: number;
  avg_final_angle_error: number;
}
```

**Ejemplo de Request**:
```json
{
  "iterations": 50,
  "vehicle_types": ["Heavy", "Standard", "Agile"],
  "threads": 4
}
```

**Ejemplo de Response**:
```json
{
  "success": true,
  "num_iterations": 50,
  "aggregate_stats": [
    {
      "vehicle_type": "Heavy",
      "total_runs": 50,
      "successes": 47,
      "success_rate": 94.0,
      "avg_arrival_time": 145.2,
      "std_arrival_time": 12.5,
      "min_arrival_time": 125.0,
      "max_arrival_time": 165.0,
      "avg_distance_traveled": 780.5,
      "std_distance_traveled": 45.2,
      "avg_final_distance": 18.3,
      "avg_final_angle_error": 1.2
    },
    {
      "vehicle_type": "Standard",
      "total_runs": 50,
      "successes": 49,
      "success_rate": 98.0,
      "avg_arrival_time": 120.5,
      "std_arrival_time": 8.3,
      "min_arrival_time": 105.0,
      "max_arrival_time": 140.0,
      "avg_distance_traveled": 720.2,
      "std_distance_traveled": 32.1,
      "avg_final_distance": 16.5,
      "avg_final_angle_error": 1.0
    },
    {
      "vehicle_type": "Agile",
      "total_runs": 50,
      "successes": 50,
      "success_rate": 100.0,
      "avg_arrival_time": 95.3,
      "std_arrival_time": 5.2,
      "min_arrival_time": 85.0,
      "max_arrival_time": 110.0,
      "avg_distance_traveled": 650.8,
      "std_distance_traveled": 25.5,
      "avg_final_distance": 14.2,
      "avg_final_angle_error": 0.8
    }
  ],
  "message": "Benchmark completed: 50 iterations across 3 vehicle types"
}
```

**Response Error** (400):
```typescript
interface ErrorResponse {
  error: string;
  details: string | null;
}
```

---

## Códigos de Estado HTTP

- **200**: Operación exitosa
- **400**: Error en los parámetros de entrada (ej: tipo de vehículo inválido)
- **500**: Error interno del servidor durante la simulación

---

## Requisitos del Frontend

El frontend debe incluir:

### 1. **Dashboard Principal**
   - Estado de la API (usando `/health`)
   - Opciones para ejecutar simulaciones o benchmarks
   - Navegación entre vistas

### 2. **Vista de Simulación**
   - Formulario para configurar parámetros de simulación:
     - Selector de tipos de vehículos (checkboxes o multi-select)
     - Inputs para dt, max_time, dimensiones del mapa, target position
     - Botón para ejecutar simulación
   - Visualización de resultados:
     - Canvas 2D mostrando el mapa y trayectorias de los vehículos
     - Colores diferentes para cada tipo de vehículo
     - Animación opcional de las trayectorias
     - Tabla con métricas de cada vehículo
   - Loading state mientras se ejecuta la simulación

### 3. **Vista de Benchmark**
   - Formulario para configurar:
     - Número de iteraciones
     - Tipos de vehículos
     - Número de threads
   - Visualización de estadísticas:
     - Gráficos de barras comparando vehículos
     - Métricas de success rate
     - Tiempos promedio de llegada con desviación estándar
     - Distancias recorridas
   - Tabla con todas las estadísticas agregadas
   - Loading state con progreso si es posible

### 4. **Componentes Reutilizables**
   - VehicleTypeSelector: Selector de tipos de vehículos
   - SimulationCanvas: Canvas para visualizar trayectorias
   - MetricsTable: Tabla para mostrar métricas
   - StatsChart: Gráficos para estadísticas de benchmark
   - LoadingSpinner: Indicador de carga

### 5. **Características Técnicas**
   - TypeScript estricto
   - React Server Components donde sea posible
   - Client Components para interactividad
   - Validación de formularios con Zod
   - Manejo de errores con toast notifications
   - Responsive design con Tailwind CSS
   - Dark mode support

### 6. **Estructura Sugerida**
```
app/
  page.tsx                 # Dashboard principal
  simulate/
    page.tsx               # Vista de simulación
  benchmark/
    page.tsx               # Vista de benchmark
components/
  ui/                      # Componentes UI básicos
  VehicleTypeSelector.tsx
  SimulationCanvas.tsx
  MetricsTable.tsx
  StatsChart.tsx
lib/
  api.ts                   # Funciones para consumir la API
  types.ts                 # TypeScript interfaces
  utils.ts                 # Utilidades
```

### 7. **Librerías Sugeridas**
   - **UI**: shadcn/ui, Tailwind CSS
   - **Forms**: React Hook Form + Zod
   - **Charts**: Recharts o Chart.js
   - **Canvas**: react-konva o HTML5 Canvas nativo
   - **HTTP**: fetch nativo o axios
   - **State**: React hooks, opcionalmente Zustand para estado global
   - **Notifications**: sonner o react-hot-toast

---

## Notas Importantes

1. **CORS**: La API tiene CORS habilitado, no habrá problemas de cross-origin.

2. **Tiempos de Respuesta**:
   - `/health`: Instantáneo
   - `/api/simulate`: 1-5 segundos (depende de max_time)
   - `/api/benchmark`: 10-60+ segundos (depende de iteraciones)

3. **Visualización del Mapa**:
   - El mapa es de coordenadas cartesianas
   - Por defecto: 1000x800 units
   - Target por defecto: (500, 700)
   - Los vehículos inician en posiciones aleatorias
   - El objetivo es llegar al target con ángulo de 90° (apuntando hacia arriba)

4. **Colores Sugeridos para Vehículos**:
   - Heavy: Rojo (#EF4444)
   - Standard: Azul (#3B82F6)
   - Agile: Verde (#10B981)

5. **Animación de Trayectorias**:
   - Usar los puntos de `trajectory[]` secuencialmente
   - Cada punto tiene timestamp `t`
   - Interpolar entre puntos para animación suave

---

## Ejemplo de Uso desde JavaScript/TypeScript

```typescript
// api.ts
const API_BASE_URL = 'http://localhost:8010';

export async function runSimulation(params: SimulationRequest): Promise<SimulationResponse> {
  const response = await fetch(`${API_BASE_URL}/api/simulate`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(params),
  });

  if (!response.ok) {
    const error: ErrorResponse = await response.json();
    throw new Error(error.details || 'Simulation failed');
  }

  return response.json();
}

export async function runBenchmark(params: BenchmarkRequest): Promise<BenchmarkResponse> {
  const response = await fetch(`${API_BASE_URL}/api/benchmark`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(params),
  });

  if (!response.ok) {
    const error: ErrorResponse = await response.json();
    throw new Error(error.details || 'Benchmark failed');
  }

  return response.json();
}

// Uso
const result = await runSimulation({
  vehicle_types: ['Standard'],
  max_time: 200
});

console.log(result.message); // "Simulation completed: 1/1 vehicles arrived successfully"
```

---

Usa esta especificación completa para generar un frontend moderno, profesional y fácil de usar en Next.js 14+ con TypeScript.
