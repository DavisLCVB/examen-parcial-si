# Sistema de Navegacion Autonoma con Logica Difusa

## Resumen Ejecutivo

Sistema de simulacion de navegacion autonoma para multiples vehiculos utilizando **logica difusa (Fuzzy Logic)** implementado en Rust. Los vehiculos deben alcanzar un objetivo especifico con un angulo de llegada determinado.

## Arquitectura del Sistema

```
+---------------------------+
|   Visualizador (egui)     |  <- Reproduccion animada
+-------------+-------------+
              |
+-------------v-------------+
|   Motor de Simulacion     |  <- Fisica 2D, dt=50ms
+-------------+-------------+
              |
+-------------v-------------+
| Controlador Fuzzy         |  <- 3 entradas, 10 reglas, 1 salida
+-------------+-------------+
              |
+-------------v-------------+
| Motor Logica Difusa       |  <- Mamdani: fuzzificacion,
| (Mamdani)                 |     inferencia, defuzzificacion
+---------------------------+
```

## Componentes Principales

### 1. Motor de Logica Difusa (`fuzzy_system/`)

| Componente | Descripcion |
|------------|-------------|
| `membership.rs` | 4 funciones de membresia: Triangular, Trapezoidal, Gaussiana, Sigmoidal |
| `sets.rs` | Conjuntos difusos con operaciones AND (min), OR (max), NOT |
| `variables.rs` | Variables linguisticas + defuzzificacion por centroide |
| `rules.rs` | Reglas IF-THEN con operadores AND/OR |
| `system.rs` | Motor de inferencia Mamdani completo |

---

## Variables Difusas y Funciones de Membresia

### Funciones de Membresia Implementadas

#### 1. Funcion Triangular
```
         b
        /\
       /  \
      /    \
     /      \
----a--------c----

mu(x) = 0                  si x <= a o x >= c
mu(x) = (x - a) / (b - a)  si a < x <= b
mu(x) = (c - x) / (c - b)  si b < x < c
```

#### 2. Funcion Trapezoidal
```
      b------c
     /        \
    /          \
   /            \
--a--------------d--

mu(x) = 0                  si x <= a o x >= d
mu(x) = (x - a) / (b - a)  si a < x < b
mu(x) = 1                  si b <= x <= c
mu(x) = (d - x) / (d - c)  si c < x < d
```

#### 3. Funcion Gaussiana
```
mu(x) = exp(-(x - centro)^2 / (2 * sigma^2))
```

#### 4. Funcion Sigmoidal
```
mu(x) = 1 / (1 + exp(-a * (x - c)))
```

---

### Variables de Entrada

#### Variable 1: `distancia_al_objetivo`
- **Universo de discurso**: [0, 1000] unidades
- **Conjuntos difusos**:

| Conjunto | Tipo | Parametros | Formula |
|----------|------|------------|---------|
| muy_cerca | Trapezoidal | (0, 0, 50, 100) | mu=1 en [0,50], decrece a 0 en [50,100] |
| media | Triangular | (80, 200, 400) | pico en 200, base [80,400] |
| lejos | Trapezoidal | (350, 500, 1000, 1000) | crece desde 350, mu=1 en [500,1000] |

#### Variable 2: `error_angular`
- **Universo de discurso**: [-pi, pi] radianes ([-180, 180] grados)
- **Convencion**: Negativo = objetivo a la izquierda, Positivo = objetivo a la derecha
- **Conjuntos difusos**:

| Conjunto | Tipo | Parametros (grados) | Descripcion |
|----------|------|---------------------|-------------|
| muy_desviado_izq | Trapezoidal | (-180, -150, -120, -70) | Desviacion extrema izquierda |
| desviado_izq | Triangular | (-90, -45, -10) | Desviacion moderada izquierda |
| alineado | Trapezoidal | (-10, -5, 5, 10) | Practicamente alineado |
| desviado_der | Triangular | (10, 45, 90) | Desviacion moderada derecha |
| muy_desviado_der | Trapezoidal | (70, 120, 150, 180) | Desviacion extrema derecha |

#### Variable 3: `velocidad_relativa`
- **Universo de discurso**: [0, 1] (normalizada)
- **Calculo**: `velocidad_actual / velocidad_maxima`
- **Conjuntos difusos**:

| Conjunto | Tipo | Parametros | Descripcion |
|----------|------|------------|-------------|
| lenta | Triangular | (0, 0, 0.3) | Menos del 30% de vel. max |
| media | Triangular | (0.2, 0.5, 0.8) | Entre 20% y 80% |
| rapida | Trapezoidal | (0.7, 1.0, 1.0, 1.0) | Mas del 70% |

---

### Variable de Salida

#### Variable: `ajuste_angular`
- **Universo de discurso**: [-m, +m] donde m = maniobrabilidad del vehiculo (grados/segundo)
- **Conjuntos difusos** (parametrizados por maniobrabilidad `m`):

| Conjunto | Tipo | Parametros | Efecto |
|----------|------|------------|--------|
| girar_izq | Triangular | (-m, -0.7m, -0.3m) | Giro fuerte a la izquierda |
| leve_izq | Triangular | (-0.4m, -0.2m, 0) | Giro suave a la izquierda |
| mantener | Triangular | (-0.1m, 0, 0.1m) | Mantener rumbo actual |
| leve_der | Triangular | (0, 0.2m, 0.4m) | Giro suave a la derecha |
| girar_der | Triangular | (0.3m, 0.7m, m) | Giro fuerte a la derecha |

---

## Base de Reglas Difusas

El sistema utiliza 10 reglas con operador AND (T-norma minimo):

| # | Antecedentes | Consecuente |
|---|--------------|-------------|
| R1 | SI distancia=lejos AND error=alineado | ENTONCES ajuste=mantener |
| R2 | SI distancia=lejos AND error=desviado_der | ENTONCES ajuste=girar_der |
| R3 | SI distancia=lejos AND error=desviado_izq | ENTONCES ajuste=girar_izq |
| R4 | SI distancia=media AND error=alineado | ENTONCES ajuste=mantener |
| R5 | SI distancia=media AND error=desviado_der | ENTONCES ajuste=leve_der |
| R6 | SI distancia=media AND error=desviado_izq | ENTONCES ajuste=leve_izq |
| R7 | SI distancia=muy_cerca AND error=alineado | ENTONCES ajuste=mantener |
| R8a | SI error=muy_desviado_izq | ENTONCES ajuste=girar_izq |
| R8b | SI error=muy_desviado_der | ENTONCES ajuste=girar_der |
| R9 | SI distancia=muy_cerca AND error=desviado_izq | ENTONCES ajuste=leve_izq |
| R10 | SI distancia=muy_cerca AND error=desviado_der | ENTONCES ajuste=leve_der |

---

## Proceso de Inferencia Difusa (Mamdani)

### Paso 1: Fuzzificacion
Convertir valores crisp a grados de membresia:
```
Para cada variable de entrada x:
    Para cada conjunto A en la variable:
        grado[A] = mu_A(x)
```

### Paso 2: Evaluacion de Reglas
```
Para cada regla R:
    activation = min(grado[antecedente_1], grado[antecedente_2], ...)  // AND
```

### Paso 3: Implicacion (Mamdani)
```
Para cada regla activada:
    mu_salida(x) = min(activation, mu_consecuente(x))
```

### Paso 4: Agregacion
```
mu_agregada(x) = max(mu_regla_1(x), mu_regla_2(x), ..., mu_regla_n(x))
```

### Paso 5: Defuzzificacion (Centroide)
```
                 integral de a hasta b de [x * mu(x) dx]
salida_crisp = ----------------------------------------
                 integral de a hasta b de [mu(x) dx]

Implementacion numerica con 1000 pasos:
    step = (b - a) / 1000
    numerador = 0
    denominador = 0
    Para x desde a hasta b con incremento step:
        numerador += x * mu_agregada(x)
        denominador += mu_agregada(x)
    salida = numerador / denominador
```

---

## Tipos de Vehiculos

| Tipo | Tamano | Maniobrabilidad (m) | Vel. Max | Aceleracion |
|------|--------|---------------------|----------|-------------|
| Heavy (Barco) | 15.0 | 20.0 deg/s | 50.0 u/s | 10.0 u/s^2 |
| Standard (Lancha) | 10.0 | 35.0 deg/s | 80.0 u/s | 20.0 u/s^2 |
| Agile (Avion) | 6.0 | 60.0 deg/s | 100.0 u/s | 30.0 u/s^2 |
| UltraAgile | 8.0 | 90.0 deg/s | 70.0 u/s | 25.0 u/s^2 |

---

## Configuracion del Mapa y Simulacion

- **Dimensiones**: 1000 x 800 unidades
- **Zona de inicio**: Franja inferior (8% de altura, y < 64)
- **Posicion inicial**: Aleatoria en zona de inicio
- **Angulo inicial**: Aleatorio [0, 2*pi]
- **Objetivo**: (500, 700) con angulo de llegada 90 grados
- **Condicion de exito**: distancia < 25 unidades AND error angular < 2 grados
- **dt**: 0.05 segundos (50ms)
- **Tiempo maximo**: 600 segundos

---

## Estrategia de Aproximacion Final

Para lograr el angulo de llegada exacto (90 grados hacia arriba):

```
Si distancia > 120 unidades:
    punto_objetivo = (target_x, target_y)

Si distancia <= 120 unidades:
    offset = 100 * (distancia / 120)^1.5
    punto_objetivo = (target_x, target_y - offset)
```

---

## Modelo Cinematico

```
x_new = x + v * cos(theta) * dt
y_new = y + v * sin(theta) * dt
theta_new = normalize(theta + ajuste_angular * dt)

donde:
    v = 0.1 * velocidad_maxima (constante)
    ajuste_angular = salida del sistema difuso
    normalize() ajusta el angulo al rango [-pi, pi]
```

---

## Metricas de Evaluacion

| Metrica | Descripcion | Unidad |
|---------|-------------|--------|
| success | Vehiculo llego al objetivo | bool |
| arrival_time | Tiempo hasta llegar | segundos |
| distance_traveled | Distancia total recorrida | unidades |
| final_distance | Distancia al objetivo al final | unidades |
| final_angle_error | Error angular final | grados |
| success_rate | Porcentaje de exitos | % |

---

## Ejecucion

```bash
# Simulacion basica
cargo run --bin navigation

# Visualizador interactivo
cargo run --bin visualizer

# Benchmark (N iteraciones)
cargo run --bin benchmark -- 100
./benchmark.sh 30 50 100

# Tests del sistema fuzzy
cargo test
```

---

## Salida del Benchmark

- `output/benchmark_Niterations.json` - Datos completos
- `output/benchmark_Niterations.csv` - Datos por iteracion (incluye configuracion aleatoria)
- `output/benchmark_Niterations_summary.csv` - Estadisticas agregadas

---

## Tecnologias

- **Lenguaje**: Rust 2024 Edition
- **Graficos**: macroquad + egui
- **Serializacion**: serde + serde_json

---

## Estadisticas del Codigo

| Metrica | Valor |
|---------|-------|
| Archivos Rust | 15 |
| Lineas de codigo | ~2500 |
| Tests unitarios | 17+ |
| Funciones de membresia | 4 tipos |
| Reglas difusas | 10 |
| Variables linguisticas | 4 |
| Conjuntos difusos | 13 |
| Pasos de integracion | 1000 |
