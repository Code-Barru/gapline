/// Dispatches on `GtfsTarget` to bind a slice `$c` to the matching feed
/// collection, then runs `$body` with that binding in scope.
///
/// The body is pasted into every match arm, so it is monomorphized per entity
/// type. `FeedInfo` is exposed as a 0-or-1 element slice via `Option::as_slice`.
///
/// ```ignore
/// let n = dispatch_slice!(target, feed, |records| records.len());
/// ```
macro_rules! dispatch_slice {
    ($target:expr, $feed:expr, |$c:ident| $body:expr) => {
        match $target {
            $crate::crud::read::GtfsTarget::Agency => {
                let $c = &$feed.agencies[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Stops => {
                let $c = &$feed.stops[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Routes => {
                let $c = &$feed.routes[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Trips => {
                let $c = &$feed.trips[..];
                $body
            }
            $crate::crud::read::GtfsTarget::StopTimes => {
                let $c = &$feed.stop_times[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Calendar => {
                let $c = &$feed.calendars[..];
                $body
            }
            $crate::crud::read::GtfsTarget::CalendarDates => {
                let $c = &$feed.calendar_dates[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Shapes => {
                let $c = &$feed.shapes[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Frequencies => {
                let $c = &$feed.frequencies[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Transfers => {
                let $c = &$feed.transfers[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Pathways => {
                let $c = &$feed.pathways[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Levels => {
                let $c = &$feed.levels[..];
                $body
            }
            $crate::crud::read::GtfsTarget::FeedInfo => {
                let $c = $feed.feed_info.as_slice();
                $body
            }
            $crate::crud::read::GtfsTarget::FareAttributes => {
                let $c = &$feed.fare_attributes[..];
                $body
            }
            $crate::crud::read::GtfsTarget::FareRules => {
                let $c = &$feed.fare_rules[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Translations => {
                let $c = &$feed.translations[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Attributions => {
                let $c = &$feed.attributions[..];
                $body
            }
        }
    };
}

/// Like [`dispatch_slice!`] but binds `$c` to `&mut [T]` for in-place mutation.
/// `FeedInfo` is exposed via `Option::as_mut_slice`.
macro_rules! dispatch_slice_mut {
    ($target:expr, $feed:expr, |$c:ident| $body:expr) => {
        match $target {
            $crate::crud::read::GtfsTarget::Agency => {
                let $c = &mut $feed.agencies[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Stops => {
                let $c = &mut $feed.stops[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Routes => {
                let $c = &mut $feed.routes[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Trips => {
                let $c = &mut $feed.trips[..];
                $body
            }
            $crate::crud::read::GtfsTarget::StopTimes => {
                let $c = &mut $feed.stop_times[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Calendar => {
                let $c = &mut $feed.calendars[..];
                $body
            }
            $crate::crud::read::GtfsTarget::CalendarDates => {
                let $c = &mut $feed.calendar_dates[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Shapes => {
                let $c = &mut $feed.shapes[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Frequencies => {
                let $c = &mut $feed.frequencies[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Transfers => {
                let $c = &mut $feed.transfers[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Pathways => {
                let $c = &mut $feed.pathways[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Levels => {
                let $c = &mut $feed.levels[..];
                $body
            }
            $crate::crud::read::GtfsTarget::FeedInfo => {
                let $c = $feed.feed_info.as_mut_slice();
                $body
            }
            $crate::crud::read::GtfsTarget::FareAttributes => {
                let $c = &mut $feed.fare_attributes[..];
                $body
            }
            $crate::crud::read::GtfsTarget::FareRules => {
                let $c = &mut $feed.fare_rules[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Translations => {
                let $c = &mut $feed.translations[..];
                $body
            }
            $crate::crud::read::GtfsTarget::Attributions => {
                let $c = &mut $feed.attributions[..];
                $body
            }
        }
    };
}

/// Dispatches a purely type-level operation. Introduces `type $T = ConcreteType`
/// as a local alias inside each arm so the body can use `$T::assoc_fn()` or
/// `some_generic::<$T>()`.
macro_rules! for_each_target_type {
    ($target:expr, |$T:ident| $body:expr) => {
        match $target {
            $crate::crud::read::GtfsTarget::Agency => {
                type $T = $crate::models::Agency;
                $body
            }
            $crate::crud::read::GtfsTarget::Stops => {
                type $T = $crate::models::Stop;
                $body
            }
            $crate::crud::read::GtfsTarget::Routes => {
                type $T = $crate::models::Route;
                $body
            }
            $crate::crud::read::GtfsTarget::Trips => {
                type $T = $crate::models::Trip;
                $body
            }
            $crate::crud::read::GtfsTarget::StopTimes => {
                type $T = $crate::models::StopTime;
                $body
            }
            $crate::crud::read::GtfsTarget::Calendar => {
                type $T = $crate::models::Calendar;
                $body
            }
            $crate::crud::read::GtfsTarget::CalendarDates => {
                type $T = $crate::models::CalendarDate;
                $body
            }
            $crate::crud::read::GtfsTarget::Shapes => {
                type $T = $crate::models::Shape;
                $body
            }
            $crate::crud::read::GtfsTarget::Frequencies => {
                type $T = $crate::models::Frequency;
                $body
            }
            $crate::crud::read::GtfsTarget::Transfers => {
                type $T = $crate::models::Transfer;
                $body
            }
            $crate::crud::read::GtfsTarget::Pathways => {
                type $T = $crate::models::Pathway;
                $body
            }
            $crate::crud::read::GtfsTarget::Levels => {
                type $T = $crate::models::Level;
                $body
            }
            $crate::crud::read::GtfsTarget::FeedInfo => {
                type $T = $crate::models::FeedInfo;
                $body
            }
            $crate::crud::read::GtfsTarget::FareAttributes => {
                type $T = $crate::models::FareAttribute;
                $body
            }
            $crate::crud::read::GtfsTarget::FareRules => {
                type $T = $crate::models::FareRule;
                $body
            }
            $crate::crud::read::GtfsTarget::Translations => {
                type $T = $crate::models::Translation;
                $body
            }
            $crate::crud::read::GtfsTarget::Attributions => {
                type $T = $crate::models::Attribution;
                $body
            }
        }
    };
}

/// Shared types and helpers for CRUD operations.
pub mod common;

/// Record creation for GTFS feeds.
pub mod create;

/// Record deletion for GTFS feeds.
pub mod delete;

/// Mini query language for filtering GTFS records.
pub mod query;

/// Read operations on GTFS feeds.
pub mod read;

/// Field-level mutation functions for GTFS records.
pub mod setters;

/// Record update for GTFS feeds.
pub mod update;

pub use create::{CreateError, CreatePlan, CreatedRecord, apply_create, validate_create};
pub use delete::{
    DeleteCascadePlan, DeleteError, DeletePlan, DeleteResult, apply_delete, validate_delete,
};
pub use query::{Filter, Query, QueryError, parse};
pub use read::{GtfsTarget, ReadError, ReadResult, read_records};
pub use update::{
    CascadePlan, UpdateError, UpdatePlan, UpdateResult, apply_update, validate_update,
};
