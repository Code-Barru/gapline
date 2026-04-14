//! FK rule: `fare_rules.fare_id` → `fare_attributes.fare_id`.

impl_fk_rule! {
    FareRulesFareFkRule,
    child_file: "fare_rules.txt",
    child: feed.fare_rules as fr,
    child_fk: fare_id (required),
    parent_file: "fare_attributes.txt",
    parent: feed.fare_attributes,
    parent_pk: fare_id (required),
    parent_entity: "fare",
}
