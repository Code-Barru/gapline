use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationType {
    StopOrPlatform = 0,
    Station = 1,
    EntranceExit = 2,
    GenericNode = 3,
    BoardingArea = 4,
}

impl LocationType {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::StopOrPlatform),
            1 => Some(Self::Station),
            2 => Some(Self::EntranceExit),
            3 => Some(Self::GenericNode),
            4 => Some(Self::BoardingArea),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteType {
    Tram,
    Subway,
    Rail,
    Bus,
    Ferry,
    CableTram,
    AerialLift,
    Funicular,
    Trolleybus,
    Monorail,
    Hvt(u16),
    Unknown(i32),
}

impl RouteType {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Tram),
            1 => Some(Self::Subway),
            2 => Some(Self::Rail),
            3 => Some(Self::Bus),
            4 => Some(Self::Ferry),
            5 => Some(Self::CableTram),
            6 => Some(Self::AerialLift),
            7 => Some(Self::Funicular),
            11 => Some(Self::Trolleybus),
            12 => Some(Self::Monorail),
            100..=1799 =>
            {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Some(Self::Hvt(val as u16))
            }
            _ => Some(Self::Unknown(val)),
        }
    }

    /// Returns the numeric value of this route type.
    #[must_use]
    pub const fn to_i32(&self) -> i32 {
        match self {
            Self::Tram => 0,
            Self::Subway => 1,
            Self::Rail => 2,
            Self::Bus => 3,
            Self::Ferry => 4,
            Self::CableTram => 5,
            Self::AerialLift => 6,
            Self::Funicular => 7,
            Self::Trolleybus => 11,
            Self::Monorail => 12,
            Self::Hvt(v) => *v as i32,
            Self::Unknown(v) => *v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PickupType {
    Regular = 0,
    NoPickup = 1,
    PhoneAgency = 2,
    CoordinateWithDriver = 3,
}

impl PickupType {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Regular),
            1 => Some(Self::NoPickup),
            2 => Some(Self::PhoneAgency),
            3 => Some(Self::CoordinateWithDriver),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropOffType {
    Regular = 0,
    NoDropOff = 1,
    PhoneAgency = 2,
    CoordinateWithDriver = 3,
}

impl DropOffType {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Regular),
            1 => Some(Self::NoDropOff),
            2 => Some(Self::PhoneAgency),
            3 => Some(Self::CoordinateWithDriver),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContinuousPickup {
    Continuous = 0,
    NoContinuous = 1,
    PhoneAgency = 2,
    CoordinateWithDriver = 3,
}

impl ContinuousPickup {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Continuous),
            1 => Some(Self::NoContinuous),
            2 => Some(Self::PhoneAgency),
            3 => Some(Self::CoordinateWithDriver),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContinuousDropOff {
    Continuous = 0,
    NoContinuous = 1,
    PhoneAgency = 2,
    CoordinateWithDriver = 3,
}

impl ContinuousDropOff {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Continuous),
            1 => Some(Self::NoContinuous),
            2 => Some(Self::PhoneAgency),
            3 => Some(Self::CoordinateWithDriver),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferType {
    Recommended = 0,
    Timed = 1,
    MinimumTime = 2,
    NotPossible = 3,
}

impl TransferType {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Recommended),
            1 => Some(Self::Timed),
            2 => Some(Self::MinimumTime),
            3 => Some(Self::NotPossible),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathwayMode {
    Walkway = 1,
    Stairs = 2,
    MovingSidewalk = 3,
    Escalator = 4,
    Elevator = 5,
    FareGate = 6,
    ExitGate = 7,
}

impl PathwayMode {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            1 => Some(Self::Walkway),
            2 => Some(Self::Stairs),
            3 => Some(Self::MovingSidewalk),
            4 => Some(Self::Escalator),
            5 => Some(Self::Elevator),
            6 => Some(Self::FareGate),
            7 => Some(Self::ExitGate),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExceptionType {
    Added = 1,
    Removed = 2,
}

impl ExceptionType {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            1 => Some(Self::Added),
            2 => Some(Self::Removed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WheelchairAccessible {
    NoInfo = 0,
    Some = 1,
    NotPossible = 2,
}

impl WheelchairAccessible {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::NoInfo),
            1 => Some(Self::Some),
            2 => Some(Self::NotPossible),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BikesAllowed {
    NoInfo = 0,
    Allowed = 1,
    NotAllowed = 2,
}

impl BikesAllowed {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::NoInfo),
            1 => Some(Self::Allowed),
            2 => Some(Self::NotAllowed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExactTimes {
    FrequencyBased = 0,
    ScheduleBased = 1,
}

impl ExactTimes {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::FrequencyBased),
            1 => Some(Self::ScheduleBased),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectionId {
    Outbound = 0,
    Inbound = 1,
}

impl DirectionId {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Outbound),
            1 => Some(Self::Inbound),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsBidirectional {
    Unidirectional = 0,
    Bidirectional = 1,
}

impl IsBidirectional {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Unidirectional),
            1 => Some(Self::Bidirectional),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Timepoint {
    Approximate = 0,
    Exact = 1,
}

impl Timepoint {
    #[must_use]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Approximate),
            1 => Some(Self::Exact),
            _ => None,
        }
    }
}
