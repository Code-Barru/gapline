//! FK rule: `transfers.from_route_id` → `routes.route_id`.

impl_fk_rule! {
    TransfersFromRouteFkRule,
    child_file: "transfers.txt",
    child: feed.transfers as t,
    child_fk: from_route_id (optional),
    parent_file: "routes.txt",
    parent: feed.routes,
    parent_pk: route_id (required),
    parent_entity: "route",
}
