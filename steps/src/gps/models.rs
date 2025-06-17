#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gps {
    /// UNIX timestamp e.g. duration after [`std::time::Instance::UNIX_EPOCH`]
    pub timestamp: std::time::Duration,
    pub latitude: f64,
    pub longitude: f64,
    /// The altitude of location in meters above the WGS84 reference ellipsoid
    pub altitude: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct Distance(f64);

impl Distance {
    pub const fn from_kilometers(km: f64) -> Self {
        Self(km)
    }

    pub const fn as_kilometers(self) -> f64 {
        self.0
    }

    pub const fn as_meters(self) -> f64 {
        self.0 * 1000.0
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Movement {
    pub distance: Distance,
    pub duration: std::time::Duration,
    pub from: Location,
    pub to: Location,
}

impl Movement {
    pub const fn speed_kmhr(&self) -> f64 {
        self.distance.as_kilometers() / (self.duration.as_secs_f64() / 60.0 / 60.0)
    }

    pub const fn is_height_corrected(&self) -> bool {
        self.from.altitude.is_some() && self.to.altitude.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    /// The altitude of location in meters above the WGS84 reference ellipsoid
    pub altitude: Option<f64>,
}

impl From<&Gps> for Location {
    fn from(
        Gps {
            timestamp: _,
            latitude,
            longitude,
            altitude,
        }: &Gps,
    ) -> Self {
        Self {
            latitude: *latitude,
            longitude: *longitude,
            altitude: *altitude,
        }
    }
}
