//! FK rule: `fare_products.fare_media_id` → `fare_media.fare_media_id`.

impl_fk_rule! {
    FareProductsMediaFkRule,
    child_file: "fare_products.txt",
    child: feed.fare_products as fp,
    child_fk: fare_media_id (optional),
    parent_file: "fare_media.txt",
    parent: feed.fare_media,
    parent_pk: fare_media_id (required),
    parent_entity: "fare media",
}
