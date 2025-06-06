use std::time::Duration;

use crate::steps::DataPoint;

const INTERPOLATION_TIME: Duration = Duration::from_millis(10);

pub fn interpolation(input: impl IntoIterator<Item = DataPoint>) -> Vec<DataPoint> {
    let input = input.into_iter().collect::<Vec<_>>();

    if input.len() < 2 {
        return Vec::new();
    }

    let start_time = input.first().cloned().expect("not empty").timestamp;

    let input = input
        .into_iter()
        .map(|mut this| {
            this.timestamp -= start_time;
            this
        })
        .collect::<Vec<_>>();

    let mut output = Vec::new();
    let mut window = input.clone();
    let mut interpolation_count = 0;

    while window.len() >= 2 {
        let time1 = window[0].timestamp;
        let time2 = window[1].timestamp;

        let number_of_points =
            (time2.as_millis() - time1.as_millis()) / INTERPOLATION_TIME.as_millis();

        for _ in 0..number_of_points {
            let interp_time = interpolation_count * INTERPOLATION_TIME;

            if time1 <= interp_time && interp_time < time2 {
                let dt = window[1].timestamp - window[0].timestamp;
                let dv = window[1].magnitude - window[0].magnitude;

                // `as` should be save as we reduce duration to difference in start
                let magnitude = (dv / dt.as_millis() as f64)
                    * (interp_time - window[0].timestamp).as_millis() as f64
                    + window[0].magnitude;

                output.push(DataPoint {
                    magnitude,
                    timestamp: interp_time,
                });
                interpolation_count += 1;
            }
        }

        window.remove(0);
    }
    output
}
