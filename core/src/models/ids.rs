use serde::{Deserialize, Serialize};

macro_rules! gtfs_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_owned())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

gtfs_id!(AgencyId);
gtfs_id!(StopId);
gtfs_id!(RouteId);
gtfs_id!(TripId);
gtfs_id!(ServiceId);
gtfs_id!(ShapeId);
gtfs_id!(FareId);
gtfs_id!(PathwayId);
gtfs_id!(LevelId);
gtfs_id!(FareMediaId);
