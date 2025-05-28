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

use time::{Duration, PrimitiveDateTime};

pub struct Report {
    pub total_resting_duration: Duration,
    pub total_exercise_duration: Duration,
    pub activity: Vec<(PrimitiveDateTime, Activity)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
        match self {
            ActivityKind::VO2 | ActivityKind::Anaerobic | ActivityKind::Aerobic => true,
            _ => false,
        }
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

pub struct Activity {
    pub heart_rate: u8,
    pub kind: ActivityKind,
}

/// Report of user activity based on heart rate.
///
/// Params:
/// - `rhr` - resting heart rate
pub fn heart_activity(
    heart_rates: impl IntoIterator<Item = (PrimitiveDateTime, u8)>,
    age: u8,
    rhr: u8,
) -> Report {
    let heart_rates = heart_rates
        .into_iter()
        .map(|(time, heart_rate)| {
            (
                time,
                Activity {
                    heart_rate,
                    kind: ActivityKind::from_rate(age, rhr, heart_rate),
                },
            )
        })
        .collect::<Vec<(PrimitiveDateTime, Activity)>>();

    let mut total_resting_duration = Duration::default();
    let mut total_exercise_duration = Duration::default();

    if heart_rates.len() < 2 {
        return Report {
            total_resting_duration,
            total_exercise_duration,
            activity: heart_rates,
        };
    }

    for i in 0..(heart_rates.len() - 1) {
        let ((first_time, activity), (second_time, _)) =
            match (heart_rates.get(i), heart_rates.get(i + 1)) {
                (Some(first), Some(second)) => (first, second),
                _ => continue,
            };

        let time_diff = *second_time - *first_time;

        match activity.kind.is_exercising() {
            true => total_exercise_duration += time_diff,
            false => total_resting_duration += time_diff,
        }
    }

    Report {
        total_resting_duration,
        total_exercise_duration,
        activity: heart_rates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use time::{Duration, macros::datetime};

    fn dt(sec: i64) -> PrimitiveDateTime {
        // Helper to create a PrimitiveDateTime at a given second offset
        datetime!(2024-01-01 00:00:00) + Duration::seconds(sec)
    }

    #[test]
    fn test_empty_input() {
        let report = heart_activity(vec![], 30, 60);
        assert_eq!(report.total_resting_duration, Duration::ZERO);
        assert_eq!(report.total_exercise_duration, Duration::ZERO);
        assert!(report.activity.is_empty());
    }

    #[test]
    fn test_single_entry() {
        let report = heart_activity(vec![(dt(0), 70)], 30, 60);
        assert_eq!(report.total_resting_duration, Duration::ZERO);
        assert_eq!(report.total_exercise_duration, Duration::ZERO);
        assert_eq!(report.activity.len(), 1);
    }

    #[test]
    fn test_all_resting() {
        let data = vec![(dt(0), 60), (dt(10), 61), (dt(20), 62)];
        let report = heart_activity(data, 40, 60);
        assert_eq!(report.total_exercise_duration, Duration::ZERO);
        assert_eq!(report.total_resting_duration, Duration::seconds(20));
        assert!(
            report
                .activity
                .iter()
                .all(|(_, a)| a.kind == ActivityKind::Resting)
        );
    }

    #[test]
    fn test_all_vo2() {
        let data = vec![(dt(0), 200), (dt(10), 201), (dt(20), 202)];
        let report = heart_activity(data, 20, 60);
        assert_eq!(report.total_resting_duration, Duration::ZERO);
        assert_eq!(report.total_exercise_duration, Duration::seconds(20));
        assert!(
            report
                .activity
                .iter()
                .all(|(_, a)| a.kind == ActivityKind::VO2)
        );
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
            (dt(0), 59),        // Resting
            (dt(10), zones[0]), // WarmUp
            (dt(20), zones[1]), // FatBurn
            (dt(30), zones[2]), // Aerobic
            (dt(40), zones[3]), // Anaerobic
            (dt(50), zones[4]), // VO2
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
        for ((_, activity), expected_kind) in report.activity.iter().zip(expected_kinds.iter()) {
            assert_eq!(&activity.kind, expected_kind);
        }

        // Check durations: 5 intervals of 10s each
        let mut expected_rest = Duration::ZERO;
        let mut expected_ex = Duration::ZERO;
        for i in 0..data.len() - 1 {
            let kind = report.activity[i].1.kind;
            let diff = Duration::seconds(10);
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
            (dt(0), 54),         // Resting
            (dt(10), aerobic),   // Aerobic
            (dt(20), anaerobic), // Anaerobic
            (dt(30), 54),        // Resting
        ];
        let report = heart_activity(data, age, rhr as u8);

        // First interval: Resting (0-10)
        // Second: Aerobic (10-20)
        // Third: Anaerobic (20-30)
        // Last: Resting (no interval after)
        assert_eq!(report.total_resting_duration, Duration::seconds(10));
        assert_eq!(report.total_exercise_duration, Duration::seconds(20));
        assert_eq!(report.activity[0].1.kind, ActivityKind::Resting);
        assert_eq!(report.activity[1].1.kind, ActivityKind::Aerobic);
        assert_eq!(report.activity[2].1.kind, ActivityKind::Anaerobic);
        assert_eq!(report.activity[3].1.kind, ActivityKind::Resting);
    }

    #[test]
    fn test_non_monotonic_timestamps() {
        let age = 35;
        let rhr = 65;
        let data = vec![(dt(20), 80), (dt(0), 60), (dt(10), 70)];
        let report = heart_activity(data, age, rhr);
        // Should not panic, but durations may be negative or nonsensical
        assert_eq!(report.activity.len(), 3);
    }

    #[test]
    fn test_large_dataset() {
        let age = 28;
        let rhr = 58;
        let mut data = Vec::new();
        for i in 0..1000 {
            let hr = if i % 2 == 0 { 60 } else { 180 };
            data.push((dt(i as i64), hr));
        }
        let report = heart_activity(data, age, rhr);
        // Should process all entries
        assert_eq!(report.activity.len(), 1000);
        // Durations should sum to 999 seconds
        assert_eq!(
            report.total_resting_duration + report.total_exercise_duration,
            Duration::seconds(999)
        );
    }
}
