use macula_passport::desks::initiate_passport::InitiatePassportV1;
use macula_passport::dossier::Command;

use crate::dossier::FfiPassport;
use crate::holder::FfiHolderKind;
use crate::FfiError;

#[uniffi::export]
impl FfiPassport {
    pub fn initiate(&self, holder_kind: FfiHolderKind, at: i64) -> Result<(), FfiError> {
        self.apply(Command::InitiatePassportV1(InitiatePassportV1 {
            holder_kind: holder_kind.into(),
            at,
        }))?;
        Ok(())
    }
}
