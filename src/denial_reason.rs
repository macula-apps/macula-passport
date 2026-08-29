/// Why a `disclose_data` attempt was denied — see
/// [`crate::desks::disclose_data`]. A refused attempt is recorded, not
/// silently dropped, per the design doc's §3; this enum is what makes
/// that record queryable later rather than a free-text reason nobody can
/// filter on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialReason {
    NoMatchingGrant,
    GrantExpired,
    GrantRevoked,
}
