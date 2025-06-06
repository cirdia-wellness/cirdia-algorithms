use std::sync::LazyLock;

use crate::steps::DataPoint;

const FILTER_LENGTH: usize = 13;

static FILTER_COEF: LazyLock<Vec<f64>> = LazyLock::new(|| {
    const FILTER_STD: f64 = 0.35;

    (0..FILTER_LENGTH)
        .map(|i| {
            std::f64::consts::E.powf(
                -0.5 * ((i as f64 - ((FILTER_LENGTH - 1) as f64) / 2.0)
                    / (FILTER_STD * ((FILTER_LENGTH - 1) as f64) / 2.0))
                    .powi(2),
            )
        })
        .collect()
});

static FILTER_SUM: LazyLock<f64> = LazyLock::new(|| FILTER_COEF.iter().sum());

pub fn filtering(input: impl IntoIterator<Item = DataPoint>) -> Vec<DataPoint> {
    let input = input.into_iter().collect::<Vec<_>>();

    if input.len() < FILTER_LENGTH {
        return Vec::new();
    }

    input
        .windows(FILTER_LENGTH)
        .map(|this| {
            let sum = this
                .iter()
                .enumerate()
                .map(|(i, this)| this.magnitude * FILTER_COEF.get(i).copied().unwrap_or_default())
                .sum::<f64>();

            DataPoint {
                magnitude: sum / *FILTER_SUM,
                timestamp: this
                    .get(FILTER_LENGTH / 2)
                    .cloned()
                    .expect("window is longer that search element")
                    .timestamp,
            }
        })
        .collect()
}
