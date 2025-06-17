# Steps by Windowed Peak Detection

Algorithm which counts steps by detection of peaks.

## Precision

Currently precision varies by person in existing dataset used to validate it. For wrist with accelerometer working in 25Hz it has following results:

| Precision | Count |
|-----------|-------|
| 80%       | 3     |
| 50%       | 6     |
| 20%       | 11    |
| <20%      | 19    |
| **Total** | **39**|

## How

As input algorithm accepts raw accelerometer data as `f64` and Unix Timestamp.

## Algorithm stages

- [Global](#global)
- [Interpolation](#interpolation-stage)
- [Filtering](#filtering-stage)
- [Scoring](#scoring-stage)
- [Detection](#detection-stage)
- [Time threshold](#time-threshold-stage)

### Global

![img](assets/algorithm/0_steps_global.png)

### Interpolation stage

![img](assets/algorithm/1_interpolation.png)

### Filtering stage

![img](assets/algorithm/2_filtering.png)

### Scoring stage

![img](assets/algorithm/3_scoring.png)

### Detection stage

![img](assets/algorithm/4_detection.png)

### Time threshold stage

![img](assets/algorithm/5_time_threshold.png)
