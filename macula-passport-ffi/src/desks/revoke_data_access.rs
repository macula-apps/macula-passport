use macula_passport::desks::revoke_data_access::RevokeDataAccessV1;
use macula_passport::dossier::Command;

use crate::dossier::FfiPassport;
use crate::{to_uuid, FfiError};

#[uniffi::export]
impl FfiPassport {
    pub fn revoke_data_access(
        &self,
        grant_id: Vec<u8>,
        acting_as: Vec<u8>,
        at: i64,
    ) -> Result<(), FfiError> {
        self.apply(Command::RevokeDataAccessV1(RevokeDataAccessV1 {
            grant_id: to_uuid(grant_id)?,
            acting_as,
            at,
        }))?;
        Ok(())
    }
}
