mod detection;
mod filtering;
mod intepolation;
mod scoring;
mod time_threshold;

#[derive(Debug, Clone)]
struct DataPoint {
    pub magnitude: f64,
    pub timestamp: std::time::Duration,
}

#[derive(Debug)]
pub struct Accelerometer {
    pub timestamp: std::time::Duration,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<Accelerometer> for DataPoint {
    fn from(Accelerometer { timestamp, x, y, z }: Accelerometer) -> Self {
        Self {
            magnitude: (x.powi(2) + y.powi(2) + z.powi(2)).sqrt(),
            timestamp,
        }
    }
}

pub fn steps_count(input: impl IntoIterator<Item = Accelerometer>) -> usize {
    let after_processing = intepolation::interpolation(input);
    let after_filter = filtering::filtering(after_processing);
    let after_scoring = scoring::scoring(after_filter);
    let after_detection = detection::detection(after_scoring);
    let after_time = time_threshold::time_threshold(after_detection);

    after_time.len()
}

#[cfg(test)]
mod tests {
    use std::{fs::File, sync::RwLock};

    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use time::{PrimitiveDateTime, UtcDateTime};

    use super::*;

    #[derive(Debug, serde::Deserialize)]
    struct TestDataCsv {
        pub timestamp: PrimitiveDateTime,
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub annotation: usize,
    }

    impl From<TestDataCsv> for Accelerometer {
        fn from(
            TestDataCsv {
                timestamp,
                x,
                y,
                z,
                annotation: _,
            }: TestDataCsv,
        ) -> Self {
            let var = timestamp.as_utc() - UtcDateTime::UNIX_EPOCH;

            Self {
                timestamp: std::time::Duration::new(
                    var.whole_seconds() as u64,
                    var.subsec_nanoseconds() as u32,
                ),
                x,
                y,
                z,
            }
        }
    }

    #[derive(Debug, serde::Serialize)]
    struct ReportRecord {
        file_name: String,
        expected: usize,
        actual: usize,
        precision: f64,
    }

    #[test]
    fn test_25() {
        let report = RwLock::new(Vec::<ReportRecord>::with_capacity(39));

        (1..40).into_par_iter().for_each(|i| {
            let file_name = format!("P{i:02}_wrist25.csv");

            let mut rdr = csv::Reader::from_reader(
                File::open(format!("assets/wrist_25hz/{file_name}")).unwrap(),
            );

            let data = rdr
                .deserialize::<TestDataCsv>()
                .filter_map(|this| this.ok())
                .collect::<Vec<_>>();

            let expected = data.iter().map(|this| this.annotation).sum::<usize>();

            let actual = steps_count(data.into_iter().map(Accelerometer::from));

            let precision = match expected < actual {
                true => expected as f64 / actual as f64,
                false => actual as f64 / expected as f64,
            };

            let precision = match precision.is_sign_negative() {
                true => precision * -1.0,
                false => precision,
            };

            report.write().unwrap().push(ReportRecord {
                file_name,
                expected,
                actual,
                precision,
            });
        });

        let mut wr = csv::Writer::from_writer(File::create("../tmp/steps_25_report.csv").unwrap());

        let mut reports = report.write().unwrap();

        reports.sort_by_key(|this| this.file_name.clone());

        reports.iter().for_each(|this| wr.serialize(this).unwrap());

        wr.flush().unwrap();
    }
}
