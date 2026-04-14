//! FK rule: `fare_attributes.agency_id` → `agency.agency_id`.

impl_fk_rule! {
    FareAttributesAgencyFkRule,
    child_file: "fare_attributes.txt",
    child: feed.fare_attributes as fa,
    child_fk: agency_id (optional),
    parent_file: "agency.txt",
    parent: feed.agencies,
    parent_pk: agency_id (optional),
    parent_entity: "agency",
}
