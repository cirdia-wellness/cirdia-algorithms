use std::time::Duration;

use crate::steps::DataPoint;

const TIME_THRESHOLD: Duration = Duration::from_millis(200);

pub fn time_threshold(input: impl IntoIterator<Item = DataPoint>) -> Vec<DataPoint> {
    let input = input.into_iter().collect::<Vec<_>>();

    if input.is_empty() {
        return Vec::new();
    }

    let mut current = input.first().cloned().expect("not empty");

    input
        .into_iter()
        .skip(1)
        .filter_map(|this| {
            if (this.timestamp - current.timestamp) > TIME_THRESHOLD {
                current = this.clone();
                return Some(this);
            }

            if this.magnitude > current.magnitude {
                current = this.clone();
            }

            None
        })
        .collect()
}
