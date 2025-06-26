use std::{collections::BTreeMap, time::Duration};

const DAY: Duration = Duration::new(86400, 0);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Record {
    pub temperature: f64,
    pub heart_rate_variability: Duration,
    pub timestamp: Duration,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PeriodStage {
    PreOvulation,
    Ovulation,
    PostOvulation,
    PeriodStart,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Period {
    pub start_timestamp: Duration,
    pub end_timestamp: Duration,
    pub kind: PeriodStage,
}

#[derive(PartialEq, PartialOrd)]
struct F64Wrapper(f64);

impl Eq for F64Wrapper {}

impl Ord for F64Wrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Algorithms assumes that all values are normal and could be compared. Anyway I think it's fine to consider NaN and NaN equal in this case
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl From<F64Wrapper> for f64 {
    fn from(F64Wrapper(value): F64Wrapper) -> Self {
        value
    }
}

enum DataPoint {
    Start {
        start_timestamp: Duration,
        end_timestamp: Duration,
    },
    MiddleUnspecified {
        start_timestamp: Duration,
        end_timestamp: Duration,
        temperature_diff: f64,
    },
    End {
        start_timestamp: Duration,
        end_timestamp: Duration,
    },
    UnknownOrCorruptedData,
}

// Q:
// Use activity to filter if yes in app or in algorithm?
// IS there any issue to compare lowest temperatures instead of avg ?
// Is sudden drop should be confirmed by next X days?
pub fn period<R: Into<Record>>(data: impl IntoIterator<Item = R>, base_temperature: f64) {
    let data = data
        .into_iter()
        .map(|this| {
            let Record {
                temperature,
                heart_rate_variability,
                timestamp,
            } = this.into();

            (timestamp, (temperature, heart_rate_variability))
        })
        .collect::<BTreeMap<_, _>>();

    if data.is_empty() {
        return;
    }

    let mut start_timestamp = data
        .first_key_value()
        .expect("Data is not empty")
        .0
        .to_owned();

    let mut period_result = Vec::new();

    let mut day_chunk;

    loop {
        let end_timestamp = start_timestamp + DAY;

        day_chunk = data
            .range(start_timestamp..end_timestamp)
            .collect::<Vec<_>>();

        if day_chunk.is_empty() {
            break;
        }

        let min_t: f64 = day_chunk
            .iter()
            .map(|this| F64Wrapper(this.1.0))
            .min()
            .expect("Not empty")
            .into();

        if min_t <= base_temperature {
            let prev_point = period_result.last();

            match prev_point {
                Some(DataPoint::MiddleUnspecified {
                    end_timestamp: prev_end_timestamp,
                    ..
                }) if end_timestamp - *prev_end_timestamp <= DAY => {
                    // TODO: Should this be day or more?
                    period_result.push(DataPoint::End {
                        start_timestamp,
                        end_timestamp,
                    })
                }
                _ => period_result.push(DataPoint::Start {
                    start_timestamp,
                    end_timestamp,
                }),
            }

            continue;
        }

        let temperature_diff = min_t - base_temperature;

        // TODO: How to process way bigger temperature diff?
        if base_temperature < min_t && temperature_diff <= 0.6 && temperature_diff >= 0.3 {
            // TODO: HOW TO CONFIRM BY HRV AND SLEEP?

            period_result.push(DataPoint::MiddleUnspecified {
                temperature_diff,
                start_timestamp,
                end_timestamp,
            });
        }

        start_timestamp = end_timestamp;
    }

    // let mut result = Vec::new();

    for (i, point) in period_result.into_iter().enumerate() {
        match point {
            DataPoint::Start {
                start_timestamp,
                end_timestamp,
            } => todo!(),
            DataPoint::MiddleUnspecified {
                start_timestamp,
                end_timestamp,
                temperature_diff,
            } => todo!(),
            DataPoint::End {
                start_timestamp,
                end_timestamp,
            } => todo!(),
            // Skip unknown case at all. Better interpolate this as other point if this suitable
            DataPoint::UnknownOrCorruptedData => continue,
        }
    }
}
