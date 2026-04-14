//! FK rule: `fare_rules.route_id` → `routes.route_id`.

impl_fk_rule! {
    FareRulesRouteFkRule,
    child_file: "fare_rules.txt",
    child: feed.fare_rules as fr,
    child_fk: route_id (optional),
    parent_file: "routes.txt",
    parent: feed.routes,
    parent_pk: route_id (required),
    parent_entity: "route",
}
