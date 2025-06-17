//! Distance between coordinates.
//!
//! If we have two coordinates we could use Haversine formula:
//!
//! ```norust
//! d=2R*sin ^ −1(√(sin^2((Φ2​−Φ1​​)/2)+cos(Φ1​)cos(Φ2​)sin^2((λ2​−λ1​​)/2)))
//! ```
//!
//! where:
//!
//! - R – Earth's radius (R = 6371 km);
//! - λ1, φ₁ – First point longitude and latitude coordinates;
//! - λ2, φ₂ – Second point longitude and latitude coordinates;
//! - d – Distance between them along Earth's surface.

mod models;

pub use models::*;

/// Radius of Earth
pub const R: f64 = 6371.0087714150598;

const WINDOW_SIZE: usize = 2;
const SPEED_THRESHOLD_KMPHR: f64 = 20.0;

pub fn movement_from_gps(data: impl IntoIterator<Item = Gps>) -> Vec<Movement> {
    let data = data.into_iter().collect::<Vec<_>>();

    data.windows(WINDOW_SIZE)
        .map(|this| {
            let first = &this[0];
            let second = &this[1];

            let distance = match (first.altitude, second.altitude) {
                (Some(altitude_1), Some(altitude_2)) => {
                    let flat_distance = haversine(
                        first.longitude,
                        first.latitude,
                        second.longitude,
                        second.latitude,
                    );

                    (flat_distance.powi(2) + ((altitude_2 - altitude_1) / 1000.0).powi(2)).sqrt()
                }
                _ => haversine(
                    first.longitude,
                    first.latitude,
                    second.longitude,
                    second.latitude,
                ),
            };

            Movement {
                distance: Distance::from_kilometers(distance),
                duration: second.timestamp - first.timestamp,
                from: Location::from(first),
                to: Location::from(second),
            }
        })
        .collect()
}

/// Calculate number of steps using data from GPS.
/// This method filters out movement if it was above threshold e.g. bicycling or driving a car.
///
/// # Params
/// - data - gps data which sorted by timestamp in asc order
/// - height - height of person in meters
/// - upper_threshold_kmphr - threshold after which algorithm stops counting this movements as running/walking and don't track as steps
pub fn steps_from_gps(
    data: impl IntoIterator<Item = Gps>,
    height: f64,
    upper_threshold_kmphr: Option<f64>,
) -> f64 {
    let steps_lenght = height * 0.41;

    let upper_threshold_kmphr = upper_threshold_kmphr.unwrap_or(SPEED_THRESHOLD_KMPHR);

    movement_from_gps(data)
        .into_iter()
        .filter_map(|this| {
            if upper_threshold_kmphr < this.speed_kmhr() {
                return None;
            }

            Some((this.distance.as_meters() / steps_lenght).floor())
        })
        .sum::<f64>()
}

/// Calculates distance from point A to point B in kilometers
fn haversine(longitude_1: f64, latitude_1: f64, longitude_2: f64, latitude_2: f64) -> f64 {
    let d_lat = (std::f64::consts::PI / 180.0) * (latitude_2 - latitude_1);
    let d_lon = (std::f64::consts::PI / 180.0) * (longitude_2 - longitude_1);

    // convert to radians
    let latitude_1 = (std::f64::consts::PI / 180.0) * latitude_1;
    let latitude_2 = (std::f64::consts::PI / 180.0) * latitude_2;

    R * (2.0
        * ((d_lat / 2.0).sin().powi(2)
            + (d_lon / 2.0).sin().powi(2) * latitude_1.cos() * latitude_2.cos())
        .sqrt()
        .asin())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_flat_large() {
        let latitude_1 = 51.5007;
        let longitude_1 = 0.1246;

        let latitude_2 = 40.6892;
        let longitude_2 = 74.0445;

        let expected = 5574.848132133367;

        let actual = haversine(longitude_1, latitude_1, longitude_2, latitude_2);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_haversine_flat_small() {
        let latitude_1 = 49.235835445219784;
        let longitude_1 = 28.48586563389628;

        let latitude_2 = 49.23297532196681;
        let longitude_2 = 28.493329182275833;

        let expected = 0.6283333685152811;

        let actual = haversine(longitude_1, latitude_1, longitude_2, latitude_2);

        assert_eq!(expected, actual);
    }

    #[test]
    fn steps() {
        let gps = [
            Gps {
                timestamp: std::time::Duration::from_secs(1000),
                latitude: 49.235835445219784,
                longitude: 28.48586563389628,
                altitude: None,
            },
            Gps {
                timestamp: std::time::Duration::from_secs(2000),
                latitude: 49.23297532196681,
                longitude: 28.493329182275833,
                altitude: None,
            },
        ];

        let expected = 806.0;

        let actual = steps_from_gps(gps, 1.9, None);

        assert_eq!(expected, actual);
    }

    #[test]
    fn steps_none_too_quick_movement() {
        let gps = [
            Gps {
                timestamp: std::time::Duration::from_secs(1000),
                latitude: 49.235835445219784,
                longitude: 28.48586563389628,
                altitude: None,
            },
            Gps {
                timestamp: std::time::Duration::from_secs(1001),
                latitude: 49.23297532196681,
                longitude: 28.493329182275833,
                altitude: None,
            },
        ];

        let expected = 0.0;

        let actual = steps_from_gps(gps, 1.9, None);

        assert_eq!(expected, actual);
    }

    #[test]
    fn steps_same_height() {
        let gps = [
            Gps {
                timestamp: std::time::Duration::from_secs(1000),
                latitude: 49.235835445219784,
                longitude: 28.48586563389628,
                altitude: Some(500.0),
            },
            Gps {
                timestamp: std::time::Duration::from_secs(2000),
                latitude: 49.23297532196681,
                longitude: 28.493329182275833,
                altitude: Some(500.0),
            },
        ];

        let expected = 806.0;

        let actual = steps_from_gps(gps, 1.9, None);

        assert_eq!(expected, actual);
    }

    #[test]
    fn steps_different_height() {
        let gps = [
            Gps {
                timestamp: std::time::Duration::from_secs(1000),
                latitude: 49.235835445219784,
                longitude: 28.48586563389628,
                altitude: Some(500.0),
            },
            Gps {
                timestamp: std::time::Duration::from_secs(2000),
                latitude: 49.23297532196681,
                longitude: 28.493329182275833,
                altitude: Some(550.0),
            },
        ];

        let expected = 809.0;

        let actual = steps_from_gps(gps, 1.9, None);

        assert_eq!(expected, actual);
    }
}
