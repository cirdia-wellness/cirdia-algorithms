use crate::steps::DataPoint;

const INITIAL_LENGTH: usize = 15;
const THRESHOLD: f64 = 1.2;

pub fn detection(input: impl IntoIterator<Item = DataPoint>) -> Vec<DataPoint> {
    let input = input.into_iter().collect::<Vec<_>>();

    let mut count = 0;
    let mut mean = 0.0;
    let mut std = 0.0;

    input
        .iter()
        .take(INITIAL_LENGTH)
        .enumerate()
        .for_each(|(index, this)| {
            let o_mean = mean;
            count += 1;
            match index {
                1 => {
                    mean = this.magnitude;
                }
                2 => {
                    mean = (mean + this.magnitude) / 2.0;
                    std = ((this.magnitude - mean).powi(2) + (o_mean - mean).powi(2)).sqrt() / 2.0
                }
                _ => {
                    mean = (this.magnitude + (count as f64 - 1.0) * mean) / count as f64;
                    std = ((count as f64 - 2.0) * std.powi(2) / (count as f64 - 1.0)
                        + (o_mean - mean).powi(2)
                        + (this.magnitude - mean).powi(2))
                    .sqrt()
                }
            }
        });

    input
        .into_iter()
        .skip(INITIAL_LENGTH)
        .filter_map(|this| {
            if (this.magnitude - mean) > std * THRESHOLD {
                return Some(this);
            }

            None
        })
        .collect()
}
