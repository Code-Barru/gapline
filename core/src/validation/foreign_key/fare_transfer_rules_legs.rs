//! FK rules: `fare_transfer_rules.from_leg_group_id` /
//! `to_leg_group_id` → `fare_leg_rules.leg_group_id`.

impl_fk_rule! {
    FareTransferRulesFromLegFkRule,
    child_file: "fare_transfer_rules.txt",
    child: feed.fare_transfer_rules as ftr,
    child_fk: from_leg_group_id (optional),
    parent_file: "fare_leg_rules.txt",
    parent: feed.fare_leg_rules,
    parent_pk: leg_group_id (optional),
    parent_entity: "leg group",
}

impl_fk_rule! {
    FareTransferRulesToLegFkRule,
    child_file: "fare_transfer_rules.txt",
    child: feed.fare_transfer_rules as ftr,
    child_fk: to_leg_group_id (optional),
    parent_file: "fare_leg_rules.txt",
    parent: feed.fare_leg_rules,
    parent_pk: leg_group_id (optional),
    parent_entity: "leg group",
}
