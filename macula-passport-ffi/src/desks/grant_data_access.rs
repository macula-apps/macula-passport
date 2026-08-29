use macula_passport::desks::grant_data_access::GrantDataAccessV1;
use macula_passport::dossier::{Command, PassportEvent};

use crate::dossier::FfiPassport;
use crate::FfiError;

#[uniffi::export]
impl FfiPassport {
    /// Returns the new grant's id (16 bytes) — pass it back to
    /// [`FfiPassport::revoke_data_access`] later.
    pub fn grant_data_access(
        &self,
        claim_type_prefix: String,
        requester: Vec<u8>,
        purpose: String,
        expires_at: Option<i64>,
        acting_as: Vec<u8>,
    ) -> Result<Vec<u8>, FfiError> {
        let event = self.apply(Command::GrantDataAccessV1(GrantDataAccessV1 {
            claim_type_prefix,
            requester,
            purpose,
            expires_at,
            acting_as,
        }))?;
        match event {
            PassportEvent::DataAccessGrantedV1(e) => Ok(e.grant.id.as_bytes().to_vec()),
            _ => unreachable!("apply(GrantDataAccessV1) always returns DataAccessGrantedV1"),
        }
    }
}
