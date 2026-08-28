/// Why a command was rejected before it ever became an event. Distinct
/// from `disclose_data` being denied — that's not a rejected command,
/// it's a successful [`crate::event::PassportEvent::DataAccessDeniedV1`],
/// see [`crate::handler`].
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
            DomainError::CustodianAlreadyAssigned => "a custodian is already assigned; use transfer, not assign",
            DomainError::NoActiveCustodian => "no active custodian to transfer from",
            DomainError::GrantNotFound => "no such grant",
            DomainError::GrantAlreadyRevoked => "grant already revoked",
        };
        write!(f, "{s}")
    }
}

impl std::error::Error for DomainError {}
