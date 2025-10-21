### Cirdia Algorithms — Overview

#### Purpose
This repository contains the signal processing and biometrics algorithms powering Cirdia wellness wearables. Algorithms are organized per metric as independent Rust crates so they can be tested and published independently.

#### Repository layout (high level)
```activity_duration/``` — identifies and aggregates activity bouts (active vs sedentary) and total active duration.

```calorie_burnt/``` — energy expenditure estimation (activity + BMR adjustments).

```heart_rate/``` — heart-rate estimation & quality metrics (HR, HRV features, artifact detection).

```period/``` — menstrual cycle tracking and period prediction.

```sleep/``` — sleep epoch classification, sleep stages, sleep summary metrics.

```steps/``` — step detection, stride counting, cadence, step confidence.

#### Design goals
- Transparency: algorithms should be readable and well-documented.

- Deterministic outputs: given same input, same outputs.

- Low resource usage: runnable on-device (embedded targets such as mobile phone chips).

- Testable: unit tests and deterministic integration tests.

#### Documentation inside each module
Each module’s README (in ```docs/``` or inside crate) should include:
1. Problem statement & expected inputs/outputs.
2. Algorithm steps (high-level + pseudo code).
3. API: public functions, expected inputs, types, units.
4. Parameters & tunables with recommended defaults and ranges.
5. Complexity and memory footprint.
6. Example usage.
7. Test plan + test vectors.
