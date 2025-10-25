# Sistema de Navegación Difusa para Vehículos

Sistema de navegación autónoma basado en lógica difusa implementado en Rust.

## Características Implementadas

### ✅ Fase 1: Sistema Base (Completado)

- **Arquitectura modular** con separación clara de responsabilidades
- **Sistema difuso completo** con 3 entradas y 10 reglas (cobertura total)
- **1 vehículo** con configuración mediante presets
- **Velocidad constante** para simplificar el control
- **Simulación física** con cinemática 2D
- **Exportación a JSON** para visualización posterior

### Módulos Implementados

```
src/
├── fuzzy_system/     ✅ Sistema difuso (corregido y testeado)
├── map/              ✅ Configuración del entorno
├── vehicle/          ✅ Tipos y características de vehículos
├── navigation/       ✅ Controlador difuso de navegación
├── simulation/       ✅ Motor de simulación física
└── bin/navigation.rs ✅ Aplicación principal
```

## Uso

### Compilar y Ejecutar

```bash
# Opción 1: Usar el script helper (ejecuta ambos pasos)
./run.sh

# Opción 2: Manual
# Paso 1: Ejecutar simulación de navegación (genera el JSON)
cargo run --bin navigation

# Paso 2: Visualizar la trayectoria (abre ventana gráfica)
cargo run --bin visualizer

# Ejecutar tests del sistema fuzzy
cargo test

# Compilar en modo release
cargo build --release
```

### Salida de Simulación

La simulación genera:
- **Consola**: Progreso de la simulación con telemetría cada 5 segundos
- **JSON**: Trayectoria completa exportada a `output/trajectory_standard.json`

Ejemplo de salida:
```
╔══════════════════════════════════════════════════════╗
║       FUZZY NAVIGATION SIMULATION STARTED           ║
╚══════════════════════════════════════════════════════╝

Vehicle Type: Standard
  - Size: 10
  - Max Speed: 80.0 units/s
  - Constant Velocity: 24.0 units/s
  - Maneuverability: 35.0°/s

✓ Vehicle arrived successfully at t=290.75s
  Distance Traveled: 6978.00 units
  Final Distance to Target: 13.78 units
```

### Visualizador 2D

El visualizador lee el archivo JSON y muestra:
- 🗺️ **Mapa** con zona de salida (verde) y objetivo (rojo)
- 🚗 **Vehículo** animado siguiendo la trayectoria
- 📈 **Trayectoria completa** con efecto de fade
- 🎮 **Controles interactivos**:
  - `SPACE`: Pausar/Reanudar
  - `← →`: Ajustar velocidad de reproducción
  - `R`: Reiniciar animación

![Visualizer Screenshot](docs/visualizer.png)

<details>
<summary>Si no se ve la ventana gráfica (Linux)</summary>

Puede que necesites instalar dependencias de desarrollo:

```bash
# Ubuntu/Debian
sudo apt install libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev

# Fedora
sudo dnf install libX11-devel libXi-devel mesa-libGL-devel alsa-lib-devel
```
</details>

## Sistema Difuso

### Entradas (3)

1. **distancia_al_objetivo** [0, 1000]
   - Sets: muy_cerca, media, lejos

2. **error_angular** [-180°, 180°]
   - Sets: alineado, desviado_izq, desviado_der, muy_desviado

3. **velocidad_relativa** [0, 1] (normalizada)
   - Sets: lenta, media, rapida

### Salidas (1)

- **ajuste_angular** [-maniobrabilidad, +maniobrabilidad]
  - Sets: girar_izq, leve_izq, mantener, leve_der, girar_der

### Reglas (10 completas)

```
R1:  SI lejos Y alineado        → mantener rumbo
R2:  SI lejos Y desviado_der    → girar derecha
R3:  SI lejos Y desviado_izq    → girar izquierda
R4:  SI media Y alineado        → mantener rumbo
R5:  SI media Y desviado_der    → giro leve derecha
R6:  SI media Y desviado_izq    → giro leve izquierda
R7:  SI muy_cerca Y alineado    → mantener rumbo
R8a: SI muy_desviado_izq        → girar izquierda fuerte
R8b: SI muy_desviado_der        → girar derecha fuerte
R9:  SI muy_cerca Y desviado_izq → giro leve izquierda
R10: SI muy_cerca Y desviado_der → giro leve derecha
```

## Configuración de Vehículos

### 4 Presets Disponibles

```rust
VehicleType::Heavy          // Pesado: lento pero robusto
VehicleType::Standard       // Estándar: balanceado
VehicleType::Agile          // Ágil: rápido y maniobrable
VehicleType::UltraAgile     // Ultra-Ágil: máxima maniobrabilidad
```

### Parámetros del Preset Standard

- Tamaño: 10.0 unidades
- Maniobrabilidad: 35°/s
- Velocidad máxima: 80.0 unidades/s
- Velocidad constante: 24.0 unidades/s (30% de max)
- Aceleración máxima: 20.0 unidades/s²

## Formato de Salida JSON

```json
{
  "vehicle_type": "Standard",
  "trajectory": [
    {
      "t": 0.05,
      "x": 242.2,
      "y": 50.3,
      "angle": 137.8,
      "velocity": 24.0,
      "distance_to_target": 698.9
    },
    ...
  ],
  "metrics": {
    "success": true,
    "arrival_time": 290.75,
    "distance_traveled": 6978.0,
    "final_angle_error": 54.07,
    "final_distance_to_target": 13.78
  }
}
```

## Visualizador Implementado

El sistema incluye un **visualizador 2D con macroquad** que muestra:

- ✅ Mapa 1000x800 con escala automática
- ✅ Zona de salida (8% inferior en verde)
- ✅ Punto objetivo (círculo rojo)
- ✅ Trayectoria completa con efecto fade
- ✅ Vehículo animado con indicador de dirección
- ✅ Información en tiempo real (tiempo, progreso, métricas)
- ✅ Controles interactivos (play/pause, velocidad, restart)
- ✅ UI con fondo semitransparente

### Características del Visualizador

- **Ventana**: 1200x900 píxeles
- **Escala**: Ajuste automático para mostrar todo el mapa
- **FPS**: ~60 fps con macroquad
- **Controles**:
  - `SPACE`: Pausar/reanudar animación
  - `← →`: Ajustar velocidad de reproducción (0.1x - 10x)
  - `R`: Reiniciar desde el inicio

## Próximas Extensiones

### Fase 2: Sistema Completo

- [ ] Múltiples vehículos (3-10)
- [ ] Detección y evitación de colisiones
- [ ] Control de velocidad variable
- [ ] Reglas para aproximación final con ángulo de llegada
- [ ] Sistema difuso expandido (5 entradas, 16 reglas)

### Fase 3: Mejoras de Visualización

- [ ] Selector de archivo JSON en la UI
- [ ] Vista de múltiples vehículos simultáneos
- [ ] Graficas de velocidad/distancia en tiempo real
- [ ] Exportar video de la simulación
- [ ] Modo debug con información de fuzzy sets

## Estructura del Proyecto

El sistema está diseñado con arquitectura modular para facilitar extensiones:

- **fuzzy_system**: Reutiliza el sistema difuso ya corregido y testeado
- **map**: Geometría y funciones auxiliares independientes
- **vehicle**: Configuración de vehículos parametrizable
- **navigation**: Controlador difuso separado del motor de simulación
- **simulation**: Bucle principal desacoplado de la lógica de control

## Notas de Implementación

- **Velocidad constante**: Simplifica el control inicial, solo se ajusta el ángulo
- **Criterio de llegada**: Basado en distancia (< 15 unidades)
- **Método de inferencia**: Mamdani con operadores min/max
- **Defuzzificación**: Centroide con integración numérica (1000 pasos)
- **Delta tiempo**: 0.05s (50ms) para simulación estable

## Correcciones Aplicadas al Sistema Fuzzy

El módulo `fuzzy_system` fue corregido para solucionar:

✅ Función Gaussiana (exp negativo)
✅ Función Sigmoidal (fórmula estándar)
✅ Operadores AND/OR (sin valores mágicos)
✅ Validación de parámetros
✅ Defuzzificación centroide verdadera
✅ Suite de 17 tests unitarios

---

**Autor**: Sistema de IA Claude Code
**Fecha**: 2025
**Lenguaje**: Rust 2024
**Licencia**: MIT
