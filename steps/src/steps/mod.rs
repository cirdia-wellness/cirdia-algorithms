mod detection;
mod filtering;
mod pre_processing;
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
    let after_processing = pre_processing::pre_processing(input);
    let after_filter = filtering::filtering(after_processing);
    let after_scoring = scoring::scoring(after_filter);
    let after_detection = detection::detection(after_scoring);
    let after_time = time_threshold::time_threshold(after_detection);

    after_time.len()
}
