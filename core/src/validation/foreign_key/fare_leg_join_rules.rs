//! FK rules for `fare_leg_join_rules.txt`:
//! - `from_network_id`, `to_network_id` → `networks.network_id`
//! - `from_stop_id`, `to_stop_id` → `stops.stop_id`

impl_fk_rule! {
    FareLegJoinRulesFromNetworkFkRule,
    child_file: "fare_leg_join_rules.txt",
    child: feed.fare_leg_join_rules as fljr,
    child_fk: from_network_id (required),
    parent_file: "networks.txt",
    parent: feed.networks,
    parent_pk: network_id (required),
    parent_entity: "network",
}

impl_fk_rule! {
    FareLegJoinRulesToNetworkFkRule,
    child_file: "fare_leg_join_rules.txt",
    child: feed.fare_leg_join_rules as fljr,
    child_fk: to_network_id (required),
    parent_file: "networks.txt",
    parent: feed.networks,
    parent_pk: network_id (required),
    parent_entity: "network",
}

impl_fk_rule! {
    FareLegJoinRulesFromStopFkRule,
    child_file: "fare_leg_join_rules.txt",
    child: feed.fare_leg_join_rules as fljr,
    child_fk: from_stop_id (optional),
    parent_file: "stops.txt",
    parent: feed.stops,
    parent_pk: stop_id (required),
    parent_entity: "stop",
}

impl_fk_rule! {
    FareLegJoinRulesToStopFkRule,
    child_file: "fare_leg_join_rules.txt",
    child: feed.fare_leg_join_rules as fljr,
    child_fk: to_stop_id (optional),
    parent_file: "stops.txt",
    parent: feed.stops,
    parent_pk: stop_id (required),
    parent_entity: "stop",
}
