//! FK rule: `fare_leg_rules.network_id` → `networks.network_id`.

impl_fk_rule! {
    FareLegRulesNetworkFkRule,
    child_file: "fare_leg_rules.txt",
    child: feed.fare_leg_rules as flr,
    child_fk: network_id (optional),
    parent_file: "networks.txt",
    parent: feed.networks,
    parent_pk: network_id (required),
    parent_entity: "network",
}
