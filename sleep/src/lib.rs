use std::{
    collections::{BTreeMap, HashSet},
    time::Duration,
};

const WINDOW_SIZE: usize = 2;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SleepMetrics {
    pub accelerometer: Accelerometer,
    pub heart_rate: u8,
    pub temperature: f64,
    pub timestamp: Duration,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Accelerometer {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Accelerometer {
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
}

#[derive(Debug, Clone)]
pub struct DetectionOptions {
    /// Allowed number of interruptions during sleeping. E.g. person rotates when sleeps and this don't mean that person woken up
    pub allowed_magnitude_jumps: usize,
    /// Threshold for magnitude which counts as movement
    pub magnitude_threshold: f64,
    /// Maximum difference for person resting heart rate and actual to consider this sleep
    pub max_heart_rate_diff: u8,
    /// How long there should be no movement to start tracking this as sleep
    pub duration_for_movement: Duration,
    /// Max delay between data points in sensors data. If delay bigger that this value sleep counting will be reset
    pub max_delay: Duration,
    pub time_to_reset_jumps: Duration,
}

impl DetectionOptions {
    pub const fn new() -> Self {
        Self {
            allowed_magnitude_jumps: 1,
            magnitude_threshold: 1.0,
            max_heart_rate_diff: 5,
            duration_for_movement: Duration::from_secs(60 * 15),
            max_delay: Duration::from_secs(60),
            time_to_reset_jumps: Duration::from_secs(60 * 15),
        }
    }

    pub const fn set_magnitude_threshold(mut self, magnitude_threshold: f64) -> Self {
        self.magnitude_threshold = magnitude_threshold;
        self
    }

    pub const fn set_duration_for_movement(
        mut self,
        duration_for_movement: std::time::Duration,
    ) -> Self {
        self.duration_for_movement = duration_for_movement;
        self
    }

    pub const fn set_allowed_magnitude_jumps(mut self, allowed_magnitude_jumps: usize) -> Self {
        self.allowed_magnitude_jumps = allowed_magnitude_jumps;
        self
    }

    pub const fn set_max_heart_rate_diff(mut self, max_heart_rate_diff: u8) -> Self {
        self.max_heart_rate_diff = max_heart_rate_diff;
        self
    }
}

impl Default for DetectionOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct DataPoint {
    pub magnitude_delta: f64,
    pub heart_rate: (u8, u8),
    // pub temperature: (f64, f64),
    pub timestamp_start: std::time::Duration,
    pub timestamp_end: std::time::Duration,
}

impl DataPoint {
    pub fn duration(&self) -> std::time::Duration {
        self.timestamp_end - self.timestamp_start
    }
}

#[derive(Debug, Clone)]
pub enum SleepPhase {
    Deep,
    Light,
    REM,
}

fn sleep_detection(
    data: impl IntoIterator<Item = SleepMetrics>,
    DetectionOptions {
        allowed_magnitude_jumps,
        magnitude_threshold,
        max_heart_rate_diff,
        duration_for_movement,
        max_delay,
        time_to_reset_jumps,
    }: DetectionOptions,
    resting_heart_rate: u8,
) -> BTreeMap<usize, DataPoint> {
    let mut threshold_buffer = std::time::Duration::default();
    let mut jumps_counter = 0;

    let data = data
        .into_iter()
        .collect::<Vec<_>>()
        .windows(WINDOW_SIZE)
        .map(|this| {
            let first = &this[0];
            let second = &this[1];

            let magnitude_delta =
                second.accelerometer.magnitude() - first.accelerometer.magnitude();

            let magnitude_delta = match magnitude_delta.is_sign_negative() {
                true => magnitude_delta * -1.0,
                false => magnitude_delta,
            };

            DataPoint {
                magnitude_delta,
                heart_rate: (first.heart_rate, second.heart_rate),
                // temperature: (first.temperature, second.temperature),
                timestamp_start: first.timestamp,
                timestamp_end: second.timestamp,
            }
        })
        .collect::<Vec<_>>();

    let mut sleep_indexes = HashSet::<usize>::new();

    let mut sleep_chunk = Vec::<usize>::new();

    let mut timer_to_reset_jumps = Duration::default();

    for (
        i,
        DataPoint {
            magnitude_delta,
            timestamp_start,
            timestamp_end,
            heart_rate: (first_hr, second_hr),
            ..
        },
    ) in data.iter().enumerate()
    {
        let time_diff = *timestamp_end - *timestamp_start;

        timer_to_reset_jumps += time_diff;
        if timer_to_reset_jumps > time_to_reset_jumps {
            jumps_counter = 0;
            timer_to_reset_jumps = Duration::default();
        }

        if *magnitude_delta > magnitude_threshold {
            jumps_counter += 1;
        }

        if jumps_counter > allowed_magnitude_jumps
            || resting_heart_rate.abs_diff(*first_hr) > max_heart_rate_diff
            || resting_heart_rate.abs_diff(*second_hr) > max_heart_rate_diff
            || time_diff > max_delay
        {
            jumps_counter = 0;
            threshold_buffer = Default::default();
            sleep_chunk.drain(..).for_each(|this| {
                sleep_indexes.insert(this);
            });

            continue;
        }

        threshold_buffer += time_diff;

        if threshold_buffer < duration_for_movement {
            continue;
        }

        sleep_chunk.push(i);
    }

    if !sleep_chunk.is_empty() {
        sleep_chunk.drain(..).for_each(|this| {
            sleep_indexes.insert(this);
        });
    }

    data.into_iter()
        .enumerate()
        .filter(|(index, _)| sleep_indexes.contains(index))
        .collect()
}

pub fn sleep_duration(
    data: impl IntoIterator<Item = SleepMetrics>,
    opt: DetectionOptions,
    resting_heart_rate: u8,
) -> std::time::Duration {
    let data = sleep_detection(data, opt, resting_heart_rate);

    let (_, time) = data.into_iter().fold(
        (0, std::time::Duration::default()),
        |(prev_index, mut time), (index, point)| {
            if prev_index == 0 {
                time += point.duration();
            }

            if prev_index + 1 == index {
                time += point.duration();
            }

            (index, time)
        },
    );

    time
}
#[cfg(test)]
mod tests {
    use super::*;

    fn make_metrics(
        times: impl IntoIterator<Item = u64>,
        mags: impl IntoIterator<Item = (f64, f64, f64)>,
        heart_rates: impl IntoIterator<Item = u8>,
        temps: impl IntoIterator<Item = f64>,
    ) -> Vec<SleepMetrics> {
        times
            .into_iter()
            .zip(mags)
            .zip(heart_rates)
            .zip(temps)
            .map(|(((t, (x, y, z)), heart_rate), temperature)| SleepMetrics {
                accelerometer: Accelerometer { x, y, z },
                heart_rate,
                temperature,
                timestamp: Duration::from_secs(t),
            })
            .collect()
    }

    #[test]
    fn detects_sleep_when_no_movement_and_heart_rate_ok() {
        let times = [0, 60, 120, 180, 240, 300];
        let mags = [(0.0, 0.0, 1.0); 6];
        let heart_rates = [60, 61, 62, 60, 61, 62];
        let temps = [36.5; 6];
        let data = make_metrics(times, mags, heart_rates, temps);

        let opt = DetectionOptions::new()
            .set_duration_for_movement(Duration::from_secs(60 * 3))
            .set_max_heart_rate_diff(5);

        let result = sleep_detection(data, opt, 60);
        assert!(!result.is_empty());
        // All points after the movement duration should be detected as sleep
        let indexes: Vec<_> = result.keys().copied().collect();
        assert!(indexes.contains(&2));
        assert!(indexes.contains(&3));
        assert!(indexes.contains(&4));
    }

    #[test]
    fn resets_on_movement_jump() {
        let times = [0, 60, 120, 180, 240, 300];
        let mags = [
            (0.0, 0.0, 1.0),
            (0.0, 0.0, 1.0),
            (10.0, 0.0, 1.0), // big jump
            (0.0, 0.0, 1.0),
            (0.0, 0.0, 1.0),
            (0.0, 0.0, 1.0),
        ];
        let heart_rates = [60; 6];
        let temps = [36.5; 6];
        let data = make_metrics(times, mags, heart_rates, temps);

        let opt = DetectionOptions::new()
            .set_duration_for_movement(Duration::from_secs(60 * 3))
            .set_magnitude_threshold(5.0)
            .set_allowed_magnitude_jumps(0);

        let result = sleep_detection(data, opt, 60);
        // Should not include indexes around the jump
        let indexes: Vec<_> = result.keys().copied().collect();
        assert!(!indexes.contains(&1));
        assert!(!indexes.contains(&2));
        assert!(!indexes.contains(&3));
    }

    #[test]
    fn resets_on_heart_rate_spike() {
        let times = [0, 60, 120, 180, 240, 300];
        let mags = [(0.0, 0.0, 1.0); 6];
        let heart_rates = [60, 61, 80, 60, 61, 62]; // spike at index 2
        let temps = [36.5; 6];
        let data = make_metrics(times, mags, heart_rates, temps);

        let opt = DetectionOptions::new()
            .set_duration_for_movement(Duration::from_secs(60 * 3))
            .set_max_heart_rate_diff(5);

        let result = sleep_detection(data, opt, 60);
        let indexes: Vec<_> = result.keys().copied().collect();
        assert!(!indexes.contains(&2));
        assert!(!indexes.contains(&3));
    }

    #[test]
    fn resets_on_large_delay() {
        let times = [0, 60, 120, 500, 560, 620];
        let mags = [(0.0, 0.0, 1.0); 6];
        let heart_rates = [60; 6];
        let temps = [36.5; 6];
        let data = make_metrics(times, mags, heart_rates, temps);

        let opt = DetectionOptions::new()
            .set_duration_for_movement(Duration::from_secs(60 * 3))
            .set_max_heart_rate_diff(5)
            .set_magnitude_threshold(1.0)
            .set_allowed_magnitude_jumps(1);

        let result = sleep_detection(data, opt, 60);
        let indexes: Vec<_> = result.keys().copied().collect();
        // Should not include indexes after the large delay
        assert!(!indexes.contains(&2));
        assert!(!indexes.contains(&3));
    }

    #[test]
    fn allows_some_jumps() {
        let times = [0, 60, 120, 180, 240, 300];
        let mags = [
            (0.0, 0.0, 1.0),
            (0.0, 0.0, 1.0),
            (2.0, 0.0, 1.0), // small jump
            (0.0, 0.0, 1.0),
            (0.0, 0.0, 1.0),
            (0.0, 0.0, 1.0),
        ];
        let heart_rates = [60; 6];
        let temps = [36.5; 6];
        let data = make_metrics(times, mags, heart_rates, temps);

        let opt = DetectionOptions::new()
            .set_duration_for_movement(Duration::from_secs(60 * 3))
            .set_magnitude_threshold(1.5)
            .set_allowed_magnitude_jumps(1);

        let result = sleep_detection(data, opt, 60);
        let indexes: Vec<_> = result.keys().copied().collect();
        // Should still detect sleep after one allowed jump
        assert!(indexes.contains(&3));
        assert!(indexes.contains(&4));
    }
}
