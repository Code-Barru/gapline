//! FK rules: `fare_leg_rules.from_timeframe_group_id` /
//! `to_timeframe_group_id` → `timeframes.timeframe_group_id`.

impl_fk_rule! {
    FareLegRulesFromTimeframeFkRule,
    child_file: "fare_leg_rules.txt",
    child: feed.fare_leg_rules as flr,
    child_fk: from_timeframe_group_id (optional),
    parent_file: "timeframes.txt",
    parent: feed.timeframes,
    parent_pk: timeframe_group_id (required),
    parent_entity: "timeframe group",
}

impl_fk_rule! {
    FareLegRulesToTimeframeFkRule,
    child_file: "fare_leg_rules.txt",
    child: feed.fare_leg_rules as flr,
    child_fk: to_timeframe_group_id (optional),
    parent_file: "timeframes.txt",
    parent: feed.timeframes,
    parent_pk: timeframe_group_id (required),
    parent_entity: "timeframe group",
}
