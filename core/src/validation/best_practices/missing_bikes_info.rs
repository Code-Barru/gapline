use crate::validation::Severity;

missing_field_rule!(
    MissingBikesInfoRule,
    rule_id = "missing_bikes_info",
    file = "trips.txt",
    collection = trips,
    field = bikes_allowed,
    severity = Severity::Info,
    message = "bikes_allowed is recommended for accessibility",
);
