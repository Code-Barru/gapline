//! FK rules: `route_networks.network_id` → `networks.network_id` and
//! `route_networks.route_id` → `routes.route_id`.

impl_fk_rule! {
    RouteNetworksNetworkFkRule,
    child_file: "route_networks.txt",
    child: feed.route_networks as rn,
    child_fk: network_id (required),
    parent_file: "networks.txt",
    parent: feed.networks,
    parent_pk: network_id (required),
    parent_entity: "network",
}

impl_fk_rule! {
    RouteNetworksRouteFkRule,
    child_file: "route_networks.txt",
    child: feed.route_networks as rn,
    child_fk: route_id (required),
    parent_file: "routes.txt",
    parent: feed.routes,
    parent_pk: route_id (required),
    parent_entity: "route",
}
