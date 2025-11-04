use std::cell::RefCell;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Acceleration {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Acceleration {
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MovementData {
    pub accelerometer: Acceleration,
    pub gyroscope: Acceleration,
}

pub fn process_raw(data: impl IntoIterator<Item = MovementData>) -> Vec<f64> {
    thread_local! {
        static FFT_PLANNER: LazyLock<RefCell<realfft::RealFftPlanner<f64>>> =
            LazyLock::new(|| RefCell::new(realfft::RealFftPlanner::<f64>::new()));
    }

    let mut data = data
        .into_iter()
        .map(
            |MovementData {
                 accelerometer,
                 gyroscope,
             }| accelerometer.magnitude() + gyroscope.magnitude(),
        )
        .collect::<Vec<_>>();

    let fft = FFT_PLANNER.with(|this| this.borrow_mut().plan_fft_forward(data.len()));

    let mut spectrum = fft.make_output_vec();

    // Seems like all errors should be impossible and more like programming errors instead of runtime
    // so I ignore them for now
    let _ = fft.process(&mut data, &mut spectrum);

    spectrum.into_iter().map(|this| this.l1_norm()).collect()
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StepSymmetryAnalyzeReport {
    pub number_of_unsymmetrical_steps: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SymmetryOptions {
    /// Magnitude above which step is considered sick or invalid
    pub magnitude: f64,
}

pub fn steps_symmetry(
    data: impl IntoIterator<Item = MovementData>,
    SymmetryOptions { magnitude }: SymmetryOptions,
) -> StepSymmetryAnalyzeReport {
    const WINDOW_SIZE: usize = 2;

    let mut number_of_unsymmetrical_steps = 0;

    process_raw(data)
        .windows(WINDOW_SIZE)
        .into_iter()
        .for_each(|this| {
            let diff = (this[0] - this[1]).abs();

            if diff > magnitude {
                number_of_unsymmetrical_steps += 1;
            }
        });

    StepSymmetryAnalyzeReport {
        number_of_unsymmetrical_steps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acceleration_magnitude() {
        let a = Acceleration {
            x: 3.0,
            y: 4.0,
            z: 0.0,
        };

        assert_eq!(a.magnitude(), 5.0);
    }

    #[test]
    fn process_raw_returns_nonempty_and_finite() {
        let data = vec![
            MovementData {
                accelerometer: Acceleration {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                gyroscope: Acceleration {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            MovementData {
                accelerometer: Acceleration {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
                gyroscope: Acceleration {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            MovementData {
                accelerometer: Acceleration {
                    x: -1.0,
                    y: 0.0,
                    z: 0.0,
                },
                gyroscope: Acceleration {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            MovementData {
                accelerometer: Acceleration {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
                gyroscope: Acceleration {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
        ];

        let out = process_raw(data);
        assert!(!out.is_empty(), "process_raw returned an empty spectrum");
        assert!(
            out.iter().all(|v| v.is_finite()),
            "spectrum contains non-finite values"
        );
    }

    #[test]
    fn steps_symmetry_large_threshold_zero_unsymmetrical() {
        // Using a very large threshold guarantees no step will be considered unsymmetrical
        let data = vec![
            MovementData {
                accelerometer: Acceleration {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                gyroscope: Acceleration {
                    x: 0.1,
                    y: 0.2,
                    z: 0.3,
                },
            },
            MovementData {
                accelerometer: Acceleration {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                gyroscope: Acceleration {
                    x: 0.1,
                    y: 0.2,
                    z: 0.3,
                },
            },
            MovementData {
                accelerometer: Acceleration {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                gyroscope: Acceleration {
                    x: 0.1,
                    y: 0.2,
                    z: 0.3,
                },
            },
        ];

        let report = steps_symmetry(
            data,
            SymmetryOptions {
                magnitude: f64::MAX / 4.0,
            },
        );
        assert_eq!(report.number_of_unsymmetrical_steps, 0);
    }
}
