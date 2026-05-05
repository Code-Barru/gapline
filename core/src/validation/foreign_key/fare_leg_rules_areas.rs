//! FK rules: `fare_leg_rules.from_area_id` / `to_area_id` → `areas.area_id`.

impl_fk_rule! {
    FareLegRulesFromAreaFkRule,
    child_file: "fare_leg_rules.txt",
    child: feed.fare_leg_rules as flr,
    child_fk: from_area_id (optional),
    parent_file: "areas.txt",
    parent: feed.areas,
    parent_pk: area_id (required),
    parent_entity: "area",
}

impl_fk_rule! {
    FareLegRulesToAreaFkRule,
    child_file: "fare_leg_rules.txt",
    child: feed.fare_leg_rules as flr,
    child_fk: to_area_id (optional),
    parent_file: "areas.txt",
    parent: feed.areas,
    parent_pk: area_id (required),
    parent_entity: "area",
}
