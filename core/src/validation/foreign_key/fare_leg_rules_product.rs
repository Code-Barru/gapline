//! FK rule: `fare_leg_rules.fare_product_id` → `fare_products.fare_product_id`.

impl_fk_rule! {
    FareLegRulesProductFkRule,
    child_file: "fare_leg_rules.txt",
    child: feed.fare_leg_rules as flr,
    child_fk: fare_product_id (required),
    parent_file: "fare_products.txt",
    parent: feed.fare_products,
    parent_pk: fare_product_id (required),
    parent_entity: "fare product",
}
