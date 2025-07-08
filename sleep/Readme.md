# Sleep

This crate implements detection of sleep based on accelerometer data and temperature.

## Params

Algorithms have optional parameters which could be customized for person to improve precision.

Params:

- Allowed magnitude jumps - allowed number of interruptions during sleeping. E.g. person rotates when sleeps and this don't mean that person woken up
- Time to reset jumps - time after which jumps counter resets (repeats)
- Magnitude threshold - threshold for magnitude which counts as movement
- Max heart rate diff - maximum difference for person resting heart rate and actual to consider this sleep
- Duration for movement - how long there should be no movement to start tracking this as sleep
- Max delay - max delay between data points in sensors data. If delay bigger that this value sleep counting will be reset

## Total time of sleep

Counting of total time slept contains two stages. This stages focus on different aspects. First stage is sleep detection which simply tracks is user sleeping and number of point. Second stage is actual counting of detected sleep periods.

### Detection

As input algorithms takes accelerometer, heart rate, temperature(currently not used) and UNIX timestamp.

1. Iterate over windows of 2 elements and find magnitude for first and second point and calculate their absolute difference;
2. Now start iterating over created list in step 1;
3. Check if it's time to reset jumps counter if so reset it to zero. We need to reset this counter after specific period of time so we count as user woke up only if there X movement over threshold during specific period of time;
4. Check if magnitude above threshold if so increment counter;
5. If counter above threshold, heart rate or time difference between points is too big you count previous points as sleep chunk and continue iteration over data;
6. If there no movement over X period of time count this as sleep point;
7. Collect all sleep point and get final result.

### Counting

As input algorithms takes accelerometer, heart rate, temperature and UNIX timestamp and passes it to [detection](#detection) stage.

1. Process raw data with [detection](#detection) stage;
2. Count difference between points timestamp if they were near. Otherwise ignore this time;
3. Get the result for duration of sleep.
