//! - THR - target heart rate
//! - MHR - maximum heart rate
//! - RHR - resting heart rate

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ActivityKind {
    VO2,
    #[default]
    Anaerobic,
    Aerobic,
    FatBurn,
    WarmUp,
}

impl ActivityKind {
    pub const fn intensity_coef(self) -> f64 {
        match self {
            ActivityKind::VO2 => 0.9,
            ActivityKind::Anaerobic => 0.8,
            ActivityKind::Aerobic => 0.7,
            ActivityKind::FatBurn => 0.6,
            ActivityKind::WarmUp => 0.5,
        }
    }
}

/// Calculate MHR for age.
///
/// # Params
/// - `age` - person age in years
#[inline]
pub const fn mhr(age: u8) -> f64 {
    207.0 - (age as f64 * 0.7)
}

#[inline]
pub const fn thr(age: u8, rhr: f64, activity: ActivityKind) -> f64 {
    ((mhr(age) - rhr) * activity.intensity_coef()) + rhr
}

/// Based on [this](https://www.thelancet.com/cms/10.1016/S2589-7500(20)30246-6/attachment/5fe9e9b1-08cc-452c-bb94-4a2f7da8316c/mmc1.pdf)
/// document and [this](https://www.mindbodygreen.com/articles/heart-rate-variability-chart) site
pub const fn average_vhr_by_age_for_male(age: u8) -> std::time::Duration {
    let ms = match age {
        ..26 => 61,
        26..31 => 56,
        31..36 => 49,
        36..41 => 43,
        41..46 => 37,
        46..51 => 34,
        51..56 => 32,
        _ => 31,
    };

    std::time::Duration::from_millis(ms)
}

/// Based on [this](https://www.thelancet.com/cms/10.1016/S2589-7500(20)30246-6/attachment/5fe9e9b1-08cc-452c-bb94-4a2f7da8316c/mmc1.pdf)
/// document and [this](https://www.mindbodygreen.com/articles/heart-rate-variability-chart) site
pub const fn average_vhr_by_age_for_female(age: u8) -> std::time::Duration {
    let ms = match age {
        ..26 => 57,
        26..31 => 53,
        31..36 => 47,
        36..41 => 42,
        41..46 => 37,
        46..51 => 34,
        51..56 => 33,
        _ => 31,
    };

    std::time::Duration::from_millis(ms)
}
