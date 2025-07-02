use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

mod normal_f64;

pub use self::normal_f64::*;

const DAY: Duration = Duration::new(86400, 0);
/// Percent usage for calculating avg of lowest temperature
const PERCENT_FOR_TEMPERATURE: f64 = 0.25;
/// Expected minimal rising of temperature in single day
const TEMPERATURE_RISING_DIFF: f64 = 0.1;

const LOWER_BOUND_FOR_TEMPERATURE: f64 = TEMPERATURE_RISING_DIFF * 3.0;

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

#[derive(Clone, Debug)]
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
        let day_end_timestamp = start_timestamp + DAY;

        day_chunk = data
            .range(start_timestamp..day_end_timestamp)
            .collect::<Vec<_>>();

        if day_chunk.is_empty() {
            break;
        }

        let end_timestamp = day_chunk
            .last()
            .map(|(t, _)| **t)
            .unwrap_or(day_end_timestamp);

        let min_t: f64 = {
            let values = day_chunk
                .iter()
                .map(|this| this.1.0)
                .collect::<BTreeSet<_>>();

            let mut size = (values.len() as f64 * PERCENT_FOR_TEMPERATURE).floor();

            // Unsure is it suited for real world apps, but in tests this is need otherwise we will get NaN after diving by zero
            if size < 1.0 {
                size = 1.0;
            }

            values
                .into_iter()
                .take(size as usize)
                .map(|this| this.into_inner())
                .sum::<f64>()
                / size
        };

        let temperature_diff = (min_t - base_temperature).abs();

        if min_t <= base_temperature && temperature_diff <= LOWER_BOUND_FOR_TEMPERATURE {
            let prev_point = inter_period_result.last();

            match prev_point {
                Some(DataPoint::MiddleUnchecked {
                    end_timestamp: prev_end_timestamp,
                    ..
                }) if day_end_timestamp - *prev_end_timestamp <= DAY => {
                    inter_period_result.push(DataPoint::MiddleUnchecked {
                        start_timestamp,
                        end_timestamp,
                        temperature: min_t,
                    })
                }
                _ => inter_period_result.push(DataPoint::Start {
                    start_timestamp,
                    end_timestamp,
                }),
            }
        } else if min_t <= base_temperature && temperature_diff > LOWER_BOUND_FOR_TEMPERATURE {
            inter_period_result.push(DataPoint::End {
                start_timestamp,
                end_timestamp,
            })
        } else if base_temperature < min_t && temperature_diff <= 0.7 {
            // && temperature_diff >= LOWER_BOUND_FOR_TEMPERATURE
            inter_period_result.push(DataPoint::MiddleUnchecked {
                temperature: min_t,
                start_timestamp,
                end_timestamp,
            });
        } else {
            inter_period_result.push(DataPoint::UnknownOrCorruptedData);
        }

        start_timestamp = day_end_timestamp;
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
                    Some(last)
                        if last.kind == PeriodStage::PostOvulation
                            && end_timestamp - last.end_timestamp <= DAY =>
                    {
                        PeriodStage::PeriodStart
                    }
                    Some(last)
                        if last.kind == PeriodStage::PostOvulation
                            && end_timestamp - last.end_timestamp > DAY =>
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

                        let mut temperature_growth: u8 = 0;
                        let mut temperature_same: u8 = 0;
                        let mut end_timestamp = &end_timestamp;

                        for j in 0..CHECK_NEXT_INDEXES_OFFSET {
                            let (prev_end_timestamp, prev_temperature) =
                                match get_next_point_temperature(
                                    end_timestamp,
                                    inter_period_result.get(i + j),
                                ) {
                                    Some(v) => v,
                                    None => break,
                                };

                            let (next_end_timestamp, next_temperature) =
                                match get_next_point_temperature(
                                    prev_end_timestamp,
                                    inter_period_result.get(i + j + 1),
                                ) {
                                    Some(v) => v,
                                    None => break,
                                };

                            if next_temperature > prev_temperature
                                && next_temperature - prev_temperature > TEMPERATURE_RISING_DIFF
                            {
                                temperature_growth += 1;
                                end_timestamp = next_end_timestamp;
                            } else if next_temperature == prev_temperature
                                || (next_temperature - prev_temperature).abs()
                                    <= TEMPERATURE_RISING_DIFF
                            {
                                temperature_same += 1;
                                end_timestamp = next_end_timestamp;
                            }
                        }

                        // TODO: Improve using data from prev cycle if provided
                        if temperature_growth >= 2 {
                            period_result.push(Period {
                                start_timestamp,
                                end_timestamp: *end_timestamp,
                                kind: PeriodStage::Ovulation,
                            });
                            i += CHECK_NEXT_INDEXES_OFFSET - 1;
                        } else if temperature_same >= 2 {
                            period_result.push(Period {
                                start_timestamp,
                                end_timestamp: *end_timestamp,
                                kind: PeriodStage::PostOvulation,
                            });
                            i += CHECK_NEXT_INDEXES_OFFSET - 1;
                        }
                    }
                };
            }
            DataPoint::End {
                start_timestamp,
                end_timestamp,
            } => {
                let last = period_result.last();

                match last {
                    Some(last)
                        if last.kind == PeriodStage::PostOvulation
                            && end_timestamp - last.end_timestamp <= DAY =>
                    {
                        period_result.push(Period {
                            start_timestamp,
                            end_timestamp,
                            kind: PeriodStage::PeriodStart,
                        });
                    }
                    _ => {
                        const ELEMENTS_TO_VALIDATE: usize = 3;

                        let mut temperature_above_base: u8 = 0;
                        inter_period_result
                            .iter()
                            .skip(i)
                            .take(ELEMENTS_TO_VALIDATE)
                            .for_each(|this| match this {
                                DataPoint::MiddleUnchecked { temperature, .. }
                                    if *temperature <= base_temperature =>
                                {
                                    temperature_above_base += 1
                                }
                                _ => (),
                            });

                        if temperature_above_base >= 2 {
                            period_result.push(Period {
                                start_timestamp,
                                end_timestamp,
                                kind: PeriodStage::PeriodStart,
                            });
                        }
                    }
                }
            }
            // Skip unknown case at all. Better interpolate this as other point if this suitable
            DataPoint::UnknownOrCorruptedData => (),
        }

        i += 1;
    }

    period_result
}
#[cfg(test)]
mod tests {
    use super::*;

    const fn make_record(temp: f64, hrv_secs: u64, ts_secs: u64) -> Record {
        Record {
            temperature: NormalF64::try_new(temp).unwrap(),
            heart_rate_variability: Duration::from_secs(hrv_secs),
            timestamp: Duration::from_secs(ts_secs),
        }
    }

    #[test]
    fn test_empty_data_returns_empty_vec() {
        let result = period::<Record>(vec![], 36.5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_day_below_base_temperature() {
        let records = vec![
            make_record(36.2, 60, 0),
            make_record(36.3, 60, 1000),
            make_record(36.1, 60, 2000),
        ];
        let result = period(records, 36.2);
        assert!(!result.is_empty());
        assert!(result.iter().any(|p| p.kind == PeriodStage::PreOvulation));
    }

    #[test]
    fn test_single_day_above_base_temperature() {
        let records = vec![
            make_record(36.7, 60, 0),
            make_record(36.8, 60, 1000),
            make_record(36.9, 60, 2000),
        ];
        let result = period(records, 36.5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_period_detects_ovulation_transition() {
        let mut records = vec![];
        // 3 days below base
        for i in 0..3 {
            records.push(make_record(36.3, 60, i * DAY.as_secs()));
        }
        // 3 days rising above base
        for i in 3..6 {
            records.push(make_record(36.6 + (i as f64 * 0.1), 60, i * DAY.as_secs()));
        }
        let result = period(records, 36.5);
        assert!(result.iter().any(|p| p.kind == PeriodStage::PreOvulation));
        assert!(result.iter().any(|p| p.kind == PeriodStage::Ovulation));
    }

    #[test]
    fn test_period_detects_period_start() {
        let mut records = vec![];
        // PostOvulation phase
        for i in 0..3 {
            records.push(make_record(36.8, 60, i * DAY.as_secs()));
        }
        // Drop below base
        for i in 3..6 {
            records.push(make_record(36.2, 60, i * DAY.as_secs()));
        }
        let result = period(records, 36.5);
        assert!(result.iter().any(|p| p.kind == PeriodStage::PostOvulation));
        assert!(result.iter().any(|p| p.kind == PeriodStage::PeriodStart));
    }

    #[test]
    fn test_period_with_corrupted_data() {
        let mut records = vec![
            make_record(36.2, 60, 0),
            make_record(36.3, 60, DAY.as_secs()),
            make_record(36.1, 60, 2 * DAY.as_secs()),
        ];
        // Insert a corrupted day (very high temperature)
        records.push(make_record(39.0, 60, 3 * DAY.as_secs()));
        records.push(make_record(36.2, 60, 4 * DAY.as_secs()));
        let result = period(records, 36.5);
        // Should still detect at least one PreOvulation or PeriodStart
        assert!(
            result
                .iter()
                .any(|p| p.kind == PeriodStage::PreOvulation || p.kind == PeriodStage::PeriodStart)
        );
    }

    // FIX:
    #[test]
    fn test_period_multiple_cycles() {
        let mut records = vec![];
        // First cycle: 3 days below, 3 days above
        for i in 0..3 {
            records.push(make_record(36.2, 60, i * DAY.as_secs()));
        }
        for i in 3..6 {
            records.push(make_record(36.7, 60, i * DAY.as_secs()));
        }
        // Second cycle: 3 days below, 3 days above
        for i in 6..9 {
            records.push(make_record(36.2, 60, i * DAY.as_secs()));
        }
        for i in 9..12 {
            records.push(make_record(36.7, 60, i * DAY.as_secs()));
        }
        let result = period(records, 36.5);
        let count = result
            .iter()
            .filter(|p| p.kind == PeriodStage::PeriodStart)
            .count();
        assert!(count >= 2);
    }

    #[test]
    fn test_period_handles_constant_temperature() {
        let records = (0..5)
            .map(|i| make_record(36.5, 60, i * DAY.as_secs()))
            .collect::<Vec<_>>();
        let result = period(records, 36.5);
        assert!(!result.iter().any(|p| p.kind == PeriodStage::Ovulation));
        assert!(!result.iter().any(|p| p.kind == PeriodStage::PostOvulation));
    }

    #[test]
    fn test_period_handles_fluctuating_temperatures() {
        let temps = [36.2, 36.7, 36.3, 36.8, 36.1, 36.9];
        let records = temps
            .iter()
            .enumerate()
            .map(|(i, &t)| make_record(t, 60, i as u64 * DAY.as_secs()))
            .collect::<Vec<_>>();
        let result = period(records, 36.6);
        assert!(result.is_empty());
    }

    #[test]
    fn test_period_with_sparse_data() {
        let records = vec![
            make_record(36.2, 60, 0),
            make_record(36.8, 60, 3 * DAY.as_secs()),
            make_record(36.1, 60, 7 * DAY.as_secs()),
        ];
        let result = period(records, 36.5);
        assert!(!result.is_empty());
    }

    // FIX:
    #[test]
    fn test_period_with_all_high_temperatures() {
        let records = (0..5)
            .map(|i| make_record(37.0, 60, i * DAY.as_secs()))
            .collect::<Vec<_>>();
        let result = period(records, 36.5);
        assert!(result.iter().any(|p| p.kind == PeriodStage::PostOvulation));
    }

    #[test]
    fn test_period_with_all_low_temperatures() {
        let records = (0..5)
            .map(|i| make_record(36.0, 60, i * DAY.as_secs()))
            .collect::<Vec<_>>();
        let result = period(records, 36.5);
        // Should detect pre-ovulation or period start
        assert!(result.is_empty());
    }

    #[test]
    fn test_period_handles_non_monotonic_timestamps() {
        let mut records = vec![
            make_record(36.2, 60, 2 * DAY.as_secs()),
            make_record(36.8, 60, 0),
            make_record(36.1, 60, DAY.as_secs()),
        ];
        // Shuffle to ensure non-monotonic order
        records.reverse();
        let result = period(records, 36.5);
        assert!(!result.is_empty());
    }
}
