//! Forward/reverse indexes of FK relationships in a [`GtfsFeed`].

mod index;
mod types;

pub use index::IntegrityIndex;
pub use types::{EntityRef, RelationType};
