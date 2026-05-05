//! FK rule: `fare_transfer_rules.fare_product_id` → `fare_products.fare_product_id`.

impl_fk_rule! {
    FareTransferRulesProductFkRule,
    child_file: "fare_transfer_rules.txt",
    child: feed.fare_transfer_rules as ftr,
    child_fk: fare_product_id (optional),
    parent_file: "fare_products.txt",
    parent: feed.fare_products,
    parent_pk: fare_product_id (required),
    parent_entity: "fare product",
}
