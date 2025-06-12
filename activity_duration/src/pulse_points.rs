use std::time::Duration;

use crate::{Activity, ActivityKind};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PulseRecord {
    pub duration: Duration,
    pub category: PulseRateCategory,
}

impl From<(PulseRateCategory, Duration)> for PulseRecord {
    fn from((category, duration): (PulseRateCategory, Duration)) -> Self {
        Self { duration, category }
    }
}

impl From<Activity> for PulseRecord {
    fn from(Activity { kind, duration, .. }: Activity) -> Self {
        Self {
            duration,
            category: kind.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PulseRateCategory {
    Low,
    Medium,
    High,
}

impl From<ActivityKind> for PulseRateCategory {
    fn from(value: ActivityKind) -> Self {
        match value {
            ActivityKind::VO2 | ActivityKind::Anaerobic | ActivityKind::Aerobic => Self::High,
            ActivityKind::FatBurn | ActivityKind::WarmUp => Self::Medium,
            ActivityKind::Resting => Self::Low,
        }
    }
}

impl PulseRateCategory {
    pub const fn weights(self) -> f64 {
        match self {
            PulseRateCategory::Low => 1.0,
            PulseRateCategory::Medium => 1.5,
            PulseRateCategory::High => 2.0,
        }
    }
}

/// The Pulse Points metric is a consolidated score that
/// sums up weighted points that are accrued on the basis
/// of how many minutes were spent at a specific heart rate category.
pub fn pulse_points<T: Into<PulseRecord>>(heart_rates: impl IntoIterator<Item = T>) -> f64 {
    let heart_rates = heart_rates.into_iter().map(Into::into).collect::<Vec<_>>();

    let categories = [
        PulseRateCategory::Low,
        PulseRateCategory::Medium,
        PulseRateCategory::High,
    ];

    categories.into_iter().fold(0.0, |acc, pulse_category| {
        let duration = heart_rates
            .iter()
            .filter_map(|PulseRecord { duration, category }| {
                if *category == pulse_category {
                    return Some(duration);
                }

                None
            })
            .sum::<Duration>();

        acc + ((duration.as_secs_f64() / 60.0) * pulse_category.weights())
    })
}
