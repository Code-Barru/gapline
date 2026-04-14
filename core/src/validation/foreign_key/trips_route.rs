//! FK rule: `trips.route_id` → `routes.route_id`.

impl_fk_rule! {
    TripsRouteFkRule,
    child_file: "trips.txt",
    child: feed.trips as t,
    child_fk: route_id (required),
    parent_file: "routes.txt",
    parent: feed.routes,
    parent_pk: route_id (required),
    parent_entity: "route",
}
