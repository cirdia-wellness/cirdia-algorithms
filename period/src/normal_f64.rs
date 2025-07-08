const ERROR_MSG: &str = "floating number is subnormal";

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(into = "f64", try_from = "f64")
)]
pub struct NormalF64(f64);

impl NormalF64 {
    /// Try to create new normal floating point number. Returns `None` if value is subnormal
    pub const fn try_new(v: f64) -> Option<Self> {
        if v.is_subnormal() {
            return None;
        }

        Some(Self(v))
    }

    #[allow(unsafe_code)]
    pub const unsafe fn unchecked_new(v: f64) -> Self {
        Self(v)
    }

    pub const fn into_inner(self) -> f64 {
        self.0
    }
}

impl Eq for NormalF64 {}

impl Ord for NormalF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Type assumes that all values are normal and could be compared
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl TryFrom<f64> for NormalF64 {
    type Error = &'static str;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::try_new(value).ok_or(ERROR_MSG)
    }
}

impl From<NormalF64> for f64 {
    fn from(NormalF64(value): NormalF64) -> Self {
        value
    }
}

impl TryFrom<f32> for NormalF64 {
    type Error = &'static str;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::try_new(f64::from(value)).ok_or(ERROR_MSG)
    }
}

impl std::ops::Deref for NormalF64 {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<f64> for NormalF64 {
    fn as_ref(&self) -> &f64 {
        &self.0
    }
}

impl std::fmt::Display for NormalF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
