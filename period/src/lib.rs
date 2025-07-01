use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

mod normal_f64;

pub use self::normal_f64::*;

const DAY: Duration = Duration::new(86400, 0);
/// Percent usage for calculating avg of lowest temperature
const PERCENT_FOR_TEMPERATURE: f64 = 0.25;
const TEMPERATURE_RISING_DIFF: f64 = 0.1;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Record {
    pub temperature: NormalF64,
    pub heart_rate_variability: Duration,
    pub timestamp: Duration,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
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

#[derive(Clone)]
enum DataPoint {
    Start {
        start_timestamp: Duration,
        end_timestamp: Duration,
    },
    MiddleUnchecked {
        start_timestamp: Duration,
        end_timestamp: Duration,
        temperature: f64,
    },
    End {
        start_timestamp: Duration,
        end_timestamp: Duration,
    },
    UnknownOrCorruptedData,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PreviousPeriod {
    pub timestamp: Duration,
    pub cycle: u8,
}

pub fn period<R: Into<Record>>(
    data: impl IntoIterator<Item = R>,
    base_temperature: f64,
) -> Vec<Period> {
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
        return Vec::new();
    }

    let mut start_timestamp = data
        .first_key_value()
        .expect("Data is not empty")
        .0
        .to_owned();

    let mut inter_period_result = Vec::new();

    let mut day_chunk;

    loop {
        let end_timestamp = start_timestamp + DAY;

        day_chunk = data
            .range(start_timestamp..end_timestamp)
            .collect::<Vec<_>>();

        if day_chunk.is_empty() {
            break;
        }

        let min_t: f64 = {
            let values = day_chunk
                .iter()
                .map(|this| this.1.0)
                .collect::<BTreeSet<_>>();

            let size = (values.len() as f64 * PERCENT_FOR_TEMPERATURE).floor();

            values
                .into_iter()
                .take(size as usize)
                .map(|this| this.into_inner())
                .sum::<f64>()
                / size
        };

        if min_t <= base_temperature {
            let prev_point = inter_period_result.last();

            match prev_point {
                Some(DataPoint::MiddleUnchecked {
                    end_timestamp: prev_end_timestamp,
                    ..
                }) if end_timestamp - *prev_end_timestamp <= DAY => {
                    inter_period_result.push(DataPoint::End {
                        start_timestamp,
                        end_timestamp,
                    })
                }
                _ => inter_period_result.push(DataPoint::Start {
                    start_timestamp,
                    end_timestamp,
                }),
            }
        }

        let temperature_diff = min_t - base_temperature;

        if base_temperature < min_t && temperature_diff <= 0.6 && temperature_diff >= 0.3 {
            inter_period_result.push(DataPoint::MiddleUnchecked {
                temperature: min_t,
                start_timestamp,
                end_timestamp,
            });
        } else {
            inter_period_result.push(DataPoint::UnknownOrCorruptedData);
        }

        start_timestamp = end_timestamp;
    }

    let mut period_result = Vec::<Period>::new();

    let mut i = 0;
    loop {
        let point = match inter_period_result.get(i) {
            Some(p) => p.to_owned(),
            None => break,
        };

        // Notes for future. Currently method checks for last point to remove possible errors
        // This should improve precision but ideally it should not only look at last element
        // but at X elements forward and back.
        match point {
            DataPoint::Start {
                start_timestamp,
                end_timestamp,
            } => {
                let kind = match period_result.last() {
                    Some(last)
                        if last.kind == PeriodStage::PreOvulation
                            || last.kind == PeriodStage::PeriodStart =>
                    {
                        PeriodStage::PreOvulation
                    }
                    // If previous period was in conflict with current I assume that previous was correct while this
                    // day was influenced by some external factor that doesn't taken into account now
                    Some(last) => last.kind,
                    None => PeriodStage::PreOvulation,
                };

                period_result.push(Period {
                    start_timestamp,
                    end_timestamp,
                    kind,
                })
            }
            // Case for elevated temperature
            DataPoint::MiddleUnchecked {
                start_timestamp,
                end_timestamp,
                temperature,
            } => {
                let last = period_result.last();

                fn get_next_point_temperature<'a>(
                    end_timestamp: &Duration,
                    next_point: Option<&'a DataPoint>,
                ) -> Option<(&'a Duration, f64)> {
                    match next_point {
                        Some(DataPoint::MiddleUnchecked {
                            end_timestamp: next_end_timestamp,
                            temperature,
                            ..
                        }) if *next_end_timestamp - *end_timestamp <= DAY => {
                            Some((next_end_timestamp, *temperature))
                        }
                        _ => None,
                    }
                }

                match last {
                    Some(last) if last.kind == PeriodStage::Ovulation => {
                        let (first_temperature, second_temperature, end_time) = {
                            let (next_time, next_point) = match get_next_point_temperature(
                                &end_timestamp,
                                inter_period_result.get(i + 1),
                            ) {
                                Some(v) => v,
                                // Lets say time diff is > then day and we can't guess temperature correctly,
                                // so simply skip this step instead of trying to give incorrect point
                                _ => continue,
                            };
                            let (time, after_next_point) = match get_next_point_temperature(
                                next_time,
                                inter_period_result.get(i + 2),
                            ) {
                                Some(v) => v,
                                // Same as above. Better to confirm with a 2 values
                                _ => continue,
                            };

                            (next_point, after_next_point, time)
                        };

                        // Temperature is same, or small differences that could be counted as same
                        if temperature == first_temperature && temperature == second_temperature
                            || (temperature - first_temperature).abs() <= TEMPERATURE_RISING_DIFF
                                && (temperature - second_temperature).abs()
                                    <= TEMPERATURE_RISING_DIFF
                        {
                            period_result.push(Period {
                                start_timestamp,
                                end_timestamp: end_time.to_owned(),
                                kind: PeriodStage::PostOvulation,
                            });
                            i += 2;
                            continue;
                        }
                        // Lets say temperature is still growing and we didn't capture it in previous cases, this should try catch it
                        else if (temperature - first_temperature).abs() > TEMPERATURE_RISING_DIFF
                            && (first_temperature - second_temperature).abs()
                                <= TEMPERATURE_RISING_DIFF
                        {
                            period_result.push(Period {
                                start_timestamp,
                                end_timestamp,
                                kind: PeriodStage::Ovulation,
                            });
                        }
                    }
                    Some(last) if last.kind == PeriodStage::PostOvulation => {
                        let mut end_timestamp = &end_timestamp;
                        i += 1;
                        loop {
                            match inter_period_result.get(i) {
                                Some(DataPoint::MiddleUnchecked {
                                    end_timestamp: next_end_timestamp,
                                    temperature: next_temperature,
                                    ..
                                }) if (next_temperature - temperature).abs()
                                    <= TEMPERATURE_RISING_DIFF =>
                                {
                                    end_timestamp = next_end_timestamp;
                                }
                                _ => break,
                            }
                        }

                        period_result.push(Period {
                            start_timestamp,
                            end_timestamp: *end_timestamp,
                            kind: PeriodStage::PostOvulation,
                        });
                        continue;
                    }
                    _ => {
                        const CHECK_NEXT_INDEXES_OFFSET: usize = 3;

                        for j in 0..=CHECK_NEXT_INDEXES_OFFSET
                        {
                            
                        }
                    }
                };
            }
            DataPoint::End {
                start_timestamp,
                end_timestamp,
            } => todo!(),
            // Skip unknown case at all. Better interpolate this as other point if this suitable
            DataPoint::UnknownOrCorruptedData => (),
        }

        i += 1;
    }

    period_result
}
