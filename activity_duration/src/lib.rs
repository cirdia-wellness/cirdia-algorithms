//! # Activity duration
//!
//! To calculate maximum heart rate(MHR)
//! we use following formula: 207 - (age) * 0.7
//!
//! Target zones:
//!
//! - VO2 Max Zone(Max) - `0.9-1.0`
//! - Anaerobic Zone(Hard) - `0.8-0.9`
//! - Aerobic Zone(Moderate ) - `0.7-0.8`
//! - Fat Burn Zone(Light) - `0.6-0.7`
//! - Warm Up Zone(Very Light ) - `0.5-0.6`
//!
//! To calculate target:
//!
//!
//! ```notrust
//! THR = [(MHR - RHR) x %Intensity] + RHR
//! ```
//!
//! Where:
//!
//! - THR - target heart rate
//! - MHR - maximum heart rate
//! - Intensity - target zone sensitive
//! - RHR - resting heart rate
//!
//! Based on American Heart Association (AHA) [data](https://www.heart.org/en/healthy-living/fitness/fitness-basics/target-heart-rates).

use std::time::Duration;

pub mod pulse_points;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityRecord {
    pub heart_rate: u8,
    pub timestamp: Duration,
}

impl From<(Duration, u8)> for ActivityRecord {
    fn from((timestamp, heart_rate): (Duration, u8)) -> Self {
        Self {
            heart_rate,
            timestamp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Report {
    pub total_resting_duration: Duration,
    pub total_exercise_duration: Duration,
    pub activity: Vec<Activity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActivityKind {
    VO2,
    Anaerobic,
    Aerobic,
    FatBurn,
    WarmUp,
    Resting,
}

impl ActivityKind {
    pub fn is_exercising(self) -> bool {
        if matches!(
            self,
            ActivityKind::VO2 | ActivityKind::Anaerobic | ActivityKind::Aerobic
        ) {
            return true;
        }

        false
    }

    pub fn from_rate(age: u8, rhr: u8, rate: u8) -> Self {
        let mhr = 207.0 - (age as f64 * 0.7);
        let rhr = rhr as f64;

        let (max_zone, hard_zone, medium_zone, light_zone, very_light_zone) = (
            (((mhr - rhr) * 0.9) + rhr).floor(),
            (((mhr - rhr) * 0.8) + rhr).floor(),
            (((mhr - rhr) * 0.7) + rhr).floor(),
            (((mhr - rhr) * 0.6) + rhr).floor(),
            (((mhr - rhr) * 0.5) + rhr).floor(),
        );

        let kind = Self::Resting;

        let rate = rate as f64;
        if rate >= max_zone {
            return Self::VO2;
        }

        if rate >= hard_zone {
            return Self::Anaerobic;
        }

        if rate >= medium_zone {
            return Self::Aerobic;
        }

        if rate >= light_zone {
            return Self::FatBurn;
        }

        if rate >= very_light_zone {
            return Self::WarmUp;
        }

        kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Activity {
    pub heart_rate: u8,
    pub kind: ActivityKind,
    pub duration: Duration,
}

/// Report of user activity based on heart rate.
///
/// Params:
/// - `rhr` - resting heart rate
pub fn heart_activity<T: Into<ActivityRecord>>(
    heart_rates: impl IntoIterator<Item = T>,
    age: u8,
    rhr: u8,
) -> Report {
    const WINDOW_SIZE: usize = 2;

    let mut heart_rates = heart_rates
        .into_iter()
        .map(Into::into)
        .collect::<Vec<ActivityRecord>>();

    heart_rates.sort_by_key(|this| this.timestamp);

    let mut total_resting_duration = Duration::default();
    let mut total_exercise_duration = Duration::default();

    let activities = heart_rates
        .windows(WINDOW_SIZE)
        .map(|this| {
            let ActivityRecord {
                heart_rate,
                timestamp,
            } = &this[0];
            let second_activity = &this[1];

            let duration = second_activity.timestamp - *timestamp;

            let kind = ActivityKind::from_rate(age, rhr, *heart_rate);

            match kind.is_exercising() {
                true => total_exercise_duration += duration,
                false => total_resting_duration += duration,
            }

            Activity {
                heart_rate: *heart_rate,
                kind,
                duration,
            }
        })
        .collect::<Vec<_>>();

    Report {
        total_resting_duration,
        total_exercise_duration,
        activity: activities,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let report = heart_activity::<ActivityRecord>(vec![], 30, 60);
        assert_eq!(report.total_resting_duration, Duration::ZERO);
        assert_eq!(report.total_exercise_duration, Duration::ZERO);
        assert!(report.activity.is_empty());
    }

    #[test]
    fn test_single_entry() {
        let report = heart_activity(vec![(Duration::from_secs(0), 70)], 30, 60);
        assert_eq!(report.total_resting_duration, Duration::ZERO);
        assert_eq!(report.total_exercise_duration, Duration::ZERO);
        assert!(report.activity.is_empty());
    }

    #[test]
    fn test_all_resting() {
        let data = vec![
            (Duration::from_secs(0), 60),
            (Duration::from_secs(10), 61),
            (Duration::from_secs(20), 62),
        ];
        let report = heart_activity(data, 40, 60);
        assert_eq!(report.total_exercise_duration, Duration::ZERO);
        assert_eq!(report.total_resting_duration, Duration::from_secs(20));
        assert!(
            report
                .activity
                .iter()
                .all(|a| a.kind == ActivityKind::Resting)
        );
    }

    #[test]
    fn test_all_vo2() {
        let data = vec![
            (Duration::from_secs(0), 200),
            (Duration::from_secs(10), 201),
            (Duration::from_secs(20), 202),
        ];
        let report = heart_activity(data, 20, 60);
        assert_eq!(report.total_resting_duration, Duration::ZERO);
        assert_eq!(report.total_exercise_duration, Duration::from_secs(20));
        assert!(report.activity.iter().all(|a| a.kind == ActivityKind::VO2));
    }

    #[test]
    fn test_mixed_zones() {
        // Calculate thresholds for age 30, rhr 60
        let age = 30;
        let rhr = 60_f64;
        let mhr = 207.0 - (age as f64 * 0.7);
        let zones = [
            ((mhr - rhr) * 0.5 + rhr).floor() as u8, // WarmUp lower bound
            ((mhr - rhr) * 0.6 + rhr).floor() as u8, // FatBurn lower bound
            ((mhr - rhr) * 0.7 + rhr).floor() as u8, // Aerobic lower bound
            ((mhr - rhr) * 0.8 + rhr).floor() as u8, // Anaerobic lower bound
            ((mhr - rhr) * 0.9 + rhr).floor() as u8, // VO2 lower bound
        ];

        let data = vec![
            (Duration::from_secs(0), 59),        // Resting
            (Duration::from_secs(10), zones[0]), // WarmUp
            (Duration::from_secs(20), zones[1]), // FatBurn
            (Duration::from_secs(30), zones[2]), // Aerobic
            (Duration::from_secs(40), zones[3]), // Anaerobic
            (Duration::from_secs(50), zones[4]), // VO2
        ];
        let report = heart_activity(data.clone(), age, rhr as u8);

        let expected_kinds = [
            ActivityKind::Resting,
            ActivityKind::WarmUp,
            ActivityKind::FatBurn,
            ActivityKind::Aerobic,
            ActivityKind::Anaerobic,
            ActivityKind::VO2,
        ];
        for (activity, expected_kind) in report.activity.iter().zip(expected_kinds.iter()) {
            assert_eq!(&activity.kind, expected_kind);
        }

        // Check durations: 5 intervals of 10s each
        let mut expected_rest = Duration::ZERO;
        let mut expected_ex = Duration::ZERO;
        for i in 0..data.len() - 1 {
            let kind = report.activity[i].kind;
            let diff = Duration::from_secs(10);
            if kind.is_exercising() {
                expected_ex += diff;
            } else {
                expected_rest += diff;
            }
        }
        assert_eq!(report.total_resting_duration, expected_rest);
        assert_eq!(report.total_exercise_duration, expected_ex);
    }

    #[test]
    fn test_transition_between_zones() {
        let age = 25;
        let rhr = 55_f64;
        let mhr = 207.0 - (age as f64 * 0.7);
        let aerobic = ((mhr - rhr) * 0.7 + rhr).floor() as u8;
        let anaerobic = ((mhr - rhr) * 0.8 + rhr).floor() as u8;

        let data = vec![
            (Duration::from_secs(0), 54),         // Resting
            (Duration::from_secs(10), aerobic),   // Aerobic
            (Duration::from_secs(20), anaerobic), // Anaerobic
            (Duration::from_secs(30), 54),        // Resting
        ];
        let report = heart_activity(data, age, rhr as u8);

        // First interval: Resting (0-10)
        // Second: Aerobic (10-20)
        // Third: Anaerobic (20-30)
        // Last: Resting (no interval after)
        assert_eq!(report.total_resting_duration, Duration::from_secs(10));
        assert_eq!(report.total_exercise_duration, Duration::from_secs(20));
        assert_eq!(report.activity[0].kind, ActivityKind::Resting);
        assert_eq!(report.activity[1].kind, ActivityKind::Aerobic);
        assert_eq!(report.activity[2].kind, ActivityKind::Anaerobic);
    }

    #[test]
    fn test_non_monotonic_timestamps() {
        let age = 35;
        let rhr = 65;
        let data = vec![
            (Duration::from_secs(20), 80),
            (Duration::from_secs(0), 60),
            (Duration::from_secs(10), 70),
        ];
        let report = heart_activity(data, age, rhr);
        assert_eq!(report.activity.len(), 2);
    }

    #[test]
    fn test_large_dataset() {
        let age = 28;
        let rhr = 58;
        let mut data = Vec::new();
        for i in 0..1000 {
            let hr = if i % 2 == 0 { 60 } else { 180 };
            data.push((Duration::from_secs(i), hr));
        }
        let report = heart_activity(data, age, rhr);
        // Should process all entries
        assert_eq!(report.activity.len(), 999);
        assert_eq!(
            report.total_resting_duration + report.total_exercise_duration,
            Duration::from_secs(999)
        );
    }
}
