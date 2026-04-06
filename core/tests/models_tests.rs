use std::collections::HashMap;

use headway_core::models::*;

// --- IDs (CA1, CA2) ---

#[test]
fn stop_id_display() {
    let id = StopId::from("S01");
    assert_eq!(id.to_string(), "S01");
}

#[test]
fn stop_id_eq() {
    assert_eq!(StopId::from("S01"), StopId::from("S01"));
    assert_ne!(StopId::from("S01"), StopId::from("S02"));
}

#[test]
fn stop_id_hash_in_map() {
    let mut map = HashMap::new();
    map.insert(StopId::from("S01"), 42);
    assert_eq!(map.get(&StopId::from("S01")), Some(&42));
}

#[test]
fn id_from_string_and_str() {
    let a = RouteId::from("R1");
    let b = RouteId::from(String::from("R1"));
    assert_eq!(a, b);
}

#[test]
fn id_as_ref_str() {
    let id = TripId::from("T1");
    let s: &str = id.as_ref();
    assert_eq!(s, "T1");
}

// StopId("S01") == TripId("S01") ne compile pas (types différents).

// --- Types (CA3) ---

#[test]
fn latitude_debug() {
    let lat = Latitude(45.5);
    assert_eq!(format!("{lat:?}"), "Latitude(45.5)");
}

#[test]
fn gtfs_date_roundtrip() {
    let d: GtfsDate = "20250301".parse().unwrap();
    assert_eq!(d.to_string(), "20250301");
}

#[test]
fn gtfs_time_over_24h() {
    let t: GtfsTime = "25:30:00".parse().unwrap();
    assert_eq!(t.hours(), 25);
    assert_eq!(t.minutes(), 30);
    assert_eq!(t.seconds(), 0);
    assert_eq!(t.to_string(), "25:30:00");
}

#[test]
fn gtfs_time_ordering() {
    let a: GtfsTime = "08:00:00".parse().unwrap();
    let b: GtfsTime = "25:30:00".parse().unwrap();
    assert!(b > a);
}

#[test]
fn color_stores_hex() {
    let c = Color::from("00AAFF");
    assert_eq!(c.0, "00AAFF");
}

// --- Enums (CA14, CA15) ---

#[test]
fn location_type_from_i32_valid() {
    assert_eq!(LocationType::from_i32(1), Some(LocationType::Station));
}

#[test]
fn location_type_from_i32_invalid() {
    assert_eq!(LocationType::from_i32(99), None);
}

#[test]
fn route_type_standard() {
    assert_eq!(RouteType::from_i32(3), Some(RouteType::Bus));
}

#[test]
fn route_type_hvt() {
    assert_eq!(RouteType::from_i32(700), Some(RouteType::Hvt(700)));
}

#[test]
fn route_type_unknown() {
    assert_eq!(RouteType::from_i32(1800), Some(RouteType::Unknown(1800)));
    assert_eq!(RouteType::from_i32(99), Some(RouteType::Unknown(99)));
    assert_eq!(RouteType::from_i32(-1), Some(RouteType::Unknown(-1)));
    assert_eq!(RouteType::from_i32(9999), Some(RouteType::Unknown(9999)));
}

#[test]
fn pickup_drop_off_enums() {
    assert_eq!(PickupType::from_i32(0), Some(PickupType::Regular));
    assert_eq!(DropOffType::from_i32(1), Some(DropOffType::NoDropOff));
    assert_eq!(PickupType::from_i32(5), None);
}

#[test]
fn transfer_type_enum() {
    assert_eq!(TransferType::from_i32(2), Some(TransferType::MinimumTime));
    assert_eq!(TransferType::from_i32(4), None);
}

#[test]
fn pathway_mode_enum() {
    assert_eq!(PathwayMode::from_i32(1), Some(PathwayMode::Walkway));
    assert_eq!(PathwayMode::from_i32(0), None);
}

#[test]
fn exception_type_enum() {
    assert_eq!(ExceptionType::from_i32(1), Some(ExceptionType::Added));
    assert_eq!(ExceptionType::from_i32(0), None);
}

#[test]
fn binary_enums() {
    assert_eq!(DirectionId::from_i32(0), Some(DirectionId::Outbound));
    assert_eq!(
        IsBidirectional::from_i32(1),
        Some(IsBidirectional::Bidirectional)
    );
    assert_eq!(Timepoint::from_i32(1), Some(Timepoint::Exact));
    assert_eq!(ExactTimes::from_i32(0), Some(ExactTimes::FrequencyBased));
    assert_eq!(BikesAllowed::from_i32(2), Some(BikesAllowed::NotAllowed));
    assert_eq!(
        WheelchairAccessible::from_i32(1),
        Some(WheelchairAccessible::Some)
    );
}

// --- GtfsFeed (CA12, CA13) ---

#[test]
fn gtfs_feed_default() {
    let feed = GtfsFeed::default();
    assert!(feed.agencies.is_empty());
    assert!(feed.stops.is_empty());
    assert!(feed.routes.is_empty());
    assert!(feed.trips.is_empty());
    assert!(feed.stop_times.is_empty());
    assert!(feed.calendars.is_empty());
    assert!(feed.calendar_dates.is_empty());
    assert!(feed.shapes.is_empty());
    assert!(feed.frequencies.is_empty());
    assert!(feed.transfers.is_empty());
    assert!(feed.pathways.is_empty());
    assert!(feed.levels.is_empty());
    assert!(feed.feed_info.is_none());
    assert!(feed.fare_attributes.is_empty());
    assert!(feed.fare_rules.is_empty());
    assert!(feed.translations.is_empty());
    assert!(feed.attributions.is_empty());
}

// --- Records serialization (CA4, CA5) ---

#[test]
fn agency_serializes_to_json() {
    let agency = Agency {
        agency_id: Some(AgencyId::from("A1")),
        agency_name: "Test Agency".into(),
        agency_url: Url::from("https://example.com"),
        agency_timezone: Timezone::from("America/Montreal"),
        agency_lang: Some(LanguageCode::from("fr")),
        agency_phone: None,
        agency_fare_url: None,
        agency_email: None,
    };
    let json = serde_json::to_string(&agency).unwrap();
    assert!(json.contains("Test Agency"));
    assert!(json.contains("A1"));
}

#[test]
fn stop_optional_fields_omitted() {
    let stop = Stop {
        stop_id: StopId::from("S1"),
        stop_code: None,
        stop_name: Some("Main St".into()),
        tts_stop_name: None,
        stop_desc: None,
        stop_lat: Some(Latitude(45.5)),
        stop_lon: Some(Longitude(-73.6)),
        zone_id: None,
        stop_url: None,
        location_type: Some(LocationType::StopOrPlatform),
        parent_station: None,
        stop_timezone: None,
        wheelchair_boarding: None,
        level_id: None,
        platform_code: None,
    };
    let json = serde_json::to_string(&stop).unwrap();
    assert!(json.contains("Main St"));
    let roundtrip: Stop = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtrip.stop_id, StopId::from("S1"));
    assert!(roundtrip.stop_desc.is_none());
}
