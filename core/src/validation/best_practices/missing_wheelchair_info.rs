use crate::validation::Severity;

missing_field_rule!(
    MissingWheelchairStopsRule,
    rule_id = "missing_wheelchair_info",
    file = "stops.txt",
    collection = stops,
    field = wheelchair_boarding,
    severity = Severity::Warning,
    message = "wheelchair_boarding is recommended for accessibility",
);

missing_field_rule!(
    MissingWheelchairTripsRule,
    rule_id = "missing_wheelchair_info",
    file = "trips.txt",
    collection = trips,
    field = wheelchair_accessible,
    severity = Severity::Warning,
    message = "wheelchair_accessible is recommended for accessibility",
);
