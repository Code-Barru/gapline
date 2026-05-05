//! FK rule: `fare_products.rider_category_id` → `rider_categories.rider_category_id`.

impl_fk_rule! {
    FareProductsRiderFkRule,
    child_file: "fare_products.txt",
    child: feed.fare_products as fp,
    child_fk: rider_category_id (optional),
    parent_file: "rider_categories.txt",
    parent: feed.rider_categories,
    parent_pk: rider_category_id (required),
    parent_entity: "rider category",
}
