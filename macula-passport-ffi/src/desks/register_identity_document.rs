use macula_passport::desks::register_identity_document::RegisterIdentityDocumentV1;
use macula_passport::dossier::Command;

use crate::claim::FfiClaim;
use crate::dossier::FfiPassport;
use crate::FfiError;

#[uniffi::export]
impl FfiPassport {
    pub fn register_identity_document(
        &self,
        claim: FfiClaim,
        acting_as: Vec<u8>,
    ) -> Result<(), FfiError> {
        self.apply(Command::RegisterIdentityDocumentV1(RegisterIdentityDocumentV1 {
            claim: claim.into(),
            acting_as,
        }))?;
        Ok(())
    }
}
