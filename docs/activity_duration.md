### Activity Duration Module
#### Purpose

Identify periods of activity vs inactivity and produce aggregated active-duration statistics per time window (minute, hour, day). Useful for activity goals, energy estimation, and sedentary-bout detection.

#### Inputs

- ```accel``` time series (x, y, z) in g or m/s², sample rate (Hz).

- Optional: ```gyro``` or classification hints.

- Optional: ```timestamp``` (UTC ISO8601 or epoch ms).

#### Outputs

- ```bouts```: list of time ranges ```{start, end, duration_s, activity_label, confidence}```.

- ```active_duration_<period>```: numeric seconds per period (e.g., day).

- ```sustained_activity_minutes``` (e.g., minutes ≥ threshold intensity).

#### Algorithm (high-level)

1. Preprocess:

- Compute vector magnitude: ```vm = sqrt(x^2 + y^2 + z^2)```.

- Bandpass or low-pass filter to remove gravity if needed (e.g., high-pass at 0.25 Hz).

- Compute short-window RMS or variance (e.g., 1 s window).

2. Feature extraction:

- For each window (e.g., 1 s) compute ```activity_score = RMS(vm)```.

- Optionally use dynamic thresholding (rolling median + scale factor).

3. Bout detection:

- Label windows above threshold as ```active```.

- Merge consecutive active windows into bouts; apply minimum bout length (e.g., 10 s) and gap-merge logic (e.g., gaps <= 5 s).

4. Summarize:

- Aggregate durations by day and compute active_minutes, sedentary_minutes.

#### Tunable parameters

```window_size_s``` (default: 1 s)

```threshold``` (default: median + k * MAD; typical ```k``` 3)

```min_bout_seconds``` (default 10)

```merge_gap_seconds``` (default 5)

#### Complexity & memory

- Time: O(N) where N = number of samples.

- Memory: O(window_size) + O(number_of_windows).

#### Tests

- Synthetic walking vs stationary traces.

- Edge-case: constant low-amplitude vibration vs real movement.
