//! Geodesic distance helpers.
//!
//! Provides a single `haversine_meters` function used by section-7 shape and
//! distance rules. No external geodesic dependency is introduced: the formula
//! is simple enough to implement directly and its accuracy is well within the
//! validation tolerances we need.

/// Mean Earth radius in meters (WGS-84 sphere approximation).
const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Great-circle distance between two `(lat, lon)` pairs expressed in decimal
/// degrees. Returns the distance in meters.
///
/// The Haversine formula assumes a spherical Earth. Worst-case error versus
/// an ellipsoid model is ~0.5%, which is negligible for GTFS validation
/// thresholds measured in meters to hundreds of meters.
#[must_use]
pub fn haversine_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let dphi = (lat2 - lat1).to_radians();
    let dlambda = (lon2 - lon1).to_radians();

    let a = (dphi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (dlambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_M * c
}
