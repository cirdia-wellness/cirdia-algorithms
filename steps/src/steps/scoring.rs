use crate::steps::DataPoint;

const SCORING_SIZE: usize = 35;

pub fn scoring(input: impl IntoIterator<Item = DataPoint>) -> Vec<DataPoint> {
    let input = input.into_iter().collect::<Vec<_>>();

    input
        .windows(SCORING_SIZE)
        .map(|data| {
            let midpoint_index = data.len() / 2;
            let midpoint = data
                .get(midpoint_index)
                .map(|this| this.magnitude)
                .expect("Point is smaller than len");

            let diff_left = data
                .iter()
                .take(midpoint_index)
                .map(|this| midpoint - this.magnitude)
                .sum::<f64>();

            let diff_right = data
                .iter()
                .skip(midpoint_index + 1)
                .map(|this| midpoint - this.magnitude)
                .sum::<f64>();

            DataPoint {
                magnitude: (diff_right + diff_left) / ((SCORING_SIZE - 1) as f64),
                timestamp: data[midpoint_index].timestamp,
            }
        })
        .collect()
}
