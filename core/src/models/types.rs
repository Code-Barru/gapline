use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Latitude(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Longitude(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GtfsDate(pub NaiveDate);

impl Default for GtfsDate {
    fn default() -> Self {
        Self(NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 is a valid date"))
    }
}

impl fmt::Display for GtfsDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y%m%d"))
    }
}

impl FromStr for GtfsDate {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NaiveDate::parse_from_str(s, "%Y%m%d").map(Self)
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct GtfsTime {
    pub total_seconds: u32,
}

impl GtfsTime {
    #[must_use]
    pub const fn from_hms(h: u32, m: u32, s: u32) -> Self {
        Self {
            total_seconds: h * 3600 + m * 60 + s,
        }
    }

    #[must_use]
    pub const fn hours(&self) -> u32 {
        self.total_seconds / 3600
    }

    #[must_use]
    pub const fn minutes(&self) -> u32 {
        (self.total_seconds % 3600) / 60
    }

    #[must_use]
    pub const fn seconds(&self) -> u32 {
        self.total_seconds % 60
    }
}

impl fmt::Display for GtfsTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.hours(),
            self.minutes(),
            self.seconds()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid GTFS time format, expected H:MM:SS or HH:MM:SS")]
pub struct GtfsTimeParseError;

impl FromStr for GtfsTime {
    type Err = GtfsTimeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(GtfsTimeParseError);
        }
        let h: u32 = parts[0].parse().map_err(|_| GtfsTimeParseError)?;
        let m: u32 = parts[1].parse().map_err(|_| GtfsTimeParseError)?;
        let s: u32 = parts[2].parse().map_err(|_| GtfsTimeParseError)?;
        Ok(Self::from_hms(h, m, s))
    }
}

macro_rules! string_wrapper {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_owned())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

string_wrapper!(Color);
string_wrapper!(Url);
string_wrapper!(Timezone);
string_wrapper!(LanguageCode);
string_wrapper!(CurrencyCode);
string_wrapper!(Email);
string_wrapper!(Phone);
