use macula_passport::claim::Claim;

use crate::value::FfiValue;
use crate::FfiError;

#[derive(uniffi::Record, Debug, Clone)]
pub struct FfiClaim {
    pub claim_type: String,
    pub value: FfiValue,
    pub issuer: Option<String>,
    pub captured_at: i64,
    pub expires_at: Option<i64>,
}

impl From<FfiClaim> for Claim {
    fn from(c: FfiClaim) -> Self {
        Claim {
            claim_type: c.claim_type,
            value: c.value.into(),
            issuer: c.issuer,
            captured_at: c.captured_at,
            expires_at: c.expires_at,
        }
    }
}

impl TryFrom<Claim> for FfiClaim {
    type Error = FfiError;

    fn try_from(c: Claim) -> Result<Self, FfiError> {
        Ok(FfiClaim {
            claim_type: c.claim_type,
            value: FfiValue::try_from(c.value)?,
            issuer: c.issuer,
            captured_at: c.captured_at,
            expires_at: c.expires_at,
        })
    }
}
