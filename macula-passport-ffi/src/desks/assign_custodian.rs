use macula_passport::desks::assign_custodian::AssignCustodianV1;
use macula_passport::dossier::Command;

use crate::dossier::FfiPassport;
use crate::FfiError;

#[uniffi::export]
impl FfiPassport {
    pub fn assign_custodian(
        &self,
        custodian: Vec<u8>,
        reason: Option<String>,
        at: i64,
    ) -> Result<(), FfiError> {
        self.apply(Command::AssignCustodianV1(AssignCustodianV1 {
            custodian,
            reason,
            at,
        }))?;
        Ok(())
    }
}
