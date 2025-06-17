use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "lowercase")
)]
pub enum Sex {
    Male,
    Female,
}

impl From<Sex> for bool {
    fn from(value: Sex) -> Self {
        match value {
            Sex::Male => true,
            Sex::Female => false,
        }
    }
}

impl From<bool> for Sex {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Male,
            false => Self::Female,
        }
    }
}

impl From<Sex> for f64 {
    fn from(value: Sex) -> Self {
        match value {
            Sex::Male => 1.0,
            Sex::Female => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct HearRate {
    age: u8,
    resting_rate: f64,
    exercise_rate: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ComplexMet {
    pub age: u8,
    pub weight: f64,
    pub heart_rate: u8,
    pub sex: Sex,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ActivityMETKind {
    /// Slowly walking or writing
    Light,
    /// Medium speed walking, simple physical activity
    Medium,
    /// More intensive medium activity like Weight lifting
    MediumPlus,
    /// Bicycling
    Vigorous,
    /// Swimming moderately to hard
    VigorousPlus,
    /// Use heart rate
    HearRateBased(HearRate),
    Complex(ComplexMet),
    /// Custom variable which isn't included as option and `HearRateBased` don't have enough precision
    Custom(f64),
}

impl ActivityMETKind {
    pub fn met_index(&self) -> f64 {
        match self {
            Self::Light => 1.5,
            Self::Medium => 3.0,
            Self::MediumPlus => 5.0,
            Self::Vigorous => 8.0,
            Self::VigorousPlus => 10.0,
            Self::Custom(index) => *index,
            Self::HearRateBased(HearRate {
                resting_rate,
                exercise_rate,
                age,
            }) => {
                let apmhr = 220 - age;
                let hrr = apmhr as f64 - resting_rate;
                let rhr = (exercise_rate - resting_rate) / hrr;

                (rhr * 3.5) + 1.0
            }
            Self::Complex(ComplexMet {
                age,
                weight,
                heart_rate,
                sex,
            }) => {
                let sex_factor = match sex {
                    Sex::Male => 0.0,
                    Sex::Female => 6.55,
                };

                ((*heart_rate as f64 * 0.6309) + (weight * 0.1988) + (*age as f64 * 0.2017)
                    - sex_factor
                    - 55.0969)
                    / 4.184
            }
        }
    }
}

/// Simple formula to calculate burnt calories
pub fn calories_burnt_by_activity_kind(
    kind: ActivityMETKind,
    duration: Duration,
    weight: f64,
) -> f64 {
    ((duration.as_secs_f64() / 60.0) * kind.met_index() * weight) / 200.0
}

#[cfg(feature = "ml")]
pub use prediction::*;

#[cfg(feature = "ml")]
mod prediction {
    use std::path::Path;

    use linfa::traits::Predict;

    use super::*;

    #[derive(Debug, Clone, PartialEq, PartialOrd, serde::Deserialize, serde::Serialize)]
    pub struct UserInfo {
        pub gender: Sex,
        pub age: u8,
        pub height: f64,
        pub weight: f64,
        pub body_temp: f64,
        pub heart_rate: u8,
        pub duration: std::time::Duration,
    }

    pub fn calories_burnt_prediction(
        UserInfo {
            gender,
            age,
            height,
            weight,
            body_temp,
            heart_rate,
            duration,
        }: UserInfo,
        model_location: impl AsRef<Path>,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let model = serde_json::from_reader::<_, linfa_trees::DecisionTree<f64, usize>>(
            std::fs::File::open(model_location.as_ref())
                .map_err(|e| format!("Failed to open model. Reason {e}"))?,
        )
        .map_err(|e| format!("Failed to init model. Reason {e}"))?;

        let data = ndarray::Array2::from_shape_vec(
            (1, 7),
            vec![
                match gender {
                    Sex::Male => 0.0,
                    Sex::Female => 1.0,
                },
                age as f64,
                height,
                weight,
                body_temp,
                heart_rate as f64,
                (duration.as_secs_f64() / 60.0),
            ],
        )
        .expect("can't fail");

        let prediction = model.predict(&data);

        let actual_calories = prediction
            .first()
            .map(|this| *this as f64)
            .ok_or("Empty prediction")?;

        Ok(actual_calories)
    }
}
