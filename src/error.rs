/// Why a command was rejected before it ever became an event. Distinct
/// from `disclose_data` being denied — that's not a rejected command,
/// it's a successful [`crate::dossier::PassportEvent::DataAccessDeniedV1`],
/// see [`crate::handler`].
///
/// Only `NotYetInitiated` is genuinely shared, reused by 5 of the 9
/// desks; the other 5 variants are each returned by exactly one desk.
/// Kept in one flat enum anyway rather than split per-desk: `Dossier`'s
/// own event-fold logic stays centralized in one file for the same
/// reason (`crate::dossier::Dossier::apply`), and splitting a 6-variant
/// enum into 9 desk-private ones would mean `handler::handle` needing a
/// `From` impl per desk to unify them back into one `Result` type —
/// real boilerplate bought for no reader benefit, since each desk
/// already shows its own `Err(DomainError::X)` inline next to the
/// `maybe_*` function that returns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainError {
    AlreadyInitiated,
    NotYetInitiated,
    CustodianAlreadyAssigned,
    NoActiveCustodian,
    GrantNotFound,
    GrantAlreadyRevoked,
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DomainError::AlreadyInitiated => "passport already initiated",
            DomainError::NotYetInitiated => "passport not yet initiated",
            DomainError::CustodianAlreadyAssigned => {
                "a custodian is already assigned; use transfer, not assign"
            }
            DomainError::NoActiveCustodian => "no active custodian to transfer from",
            DomainError::GrantNotFound => "no such grant",
            DomainError::GrantAlreadyRevoked => "grant already revoked",
        };
        write!(f, "{s}")
    }
}

impl std::error::Error for DomainError {}
