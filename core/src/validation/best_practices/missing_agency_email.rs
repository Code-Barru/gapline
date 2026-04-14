use crate::validation::Severity;

missing_field_rule!(
    MissingAgencyEmailRule,
    rule_id = "missing_agency_email",
    file = "agency.txt",
    collection = agencies,
    field = agency_email,
    severity = Severity::Info,
    message = "agency_email is recommended for contact purposes",
);
