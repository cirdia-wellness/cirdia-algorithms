/// Simple formula to calculate virtual steps
///
/// If Heart Rate increases significantly above
/// Resting Heart Rate and movement is low i.e. steps = 0,
/// use the Metabolic Equivalent of Task (MET)
/// to estimate the number of virtual steps.
///
/// # Params
/// - `met` - metabolic equivalent of task
/// - `weight` - weight of person in kilograms
#[inline]
pub fn virtual_steps(met: f64, weight: f64) -> u64 {
    (((met * weight) * 3.5) / 3.0).floor() as u64
}
