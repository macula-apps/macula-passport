use macula_passport::denial_reason::DenialReason;

#[derive(uniffi::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiDenialReason {
    NoMatchingGrant,
    GrantExpired,
    GrantRevoked,
}

impl From<DenialReason> for FfiDenialReason {
    fn from(r: DenialReason) -> Self {
        match r {
            DenialReason::NoMatchingGrant => FfiDenialReason::NoMatchingGrant,
            DenialReason::GrantExpired => FfiDenialReason::GrantExpired,
            DenialReason::GrantRevoked => FfiDenialReason::GrantRevoked,
        }
    }
}
