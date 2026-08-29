use macula_passport::desks::transfer_custodianship::TransferCustodianshipV1;
use macula_passport::dossier::Command;

use crate::dossier::FfiPassport;
use crate::FfiError;

#[uniffi::export]
impl FfiPassport {
    pub fn transfer_custodianship(&self, to: Option<Vec<u8>>, at: i64) -> Result<(), FfiError> {
        self.apply(Command::TransferCustodianshipV1(TransferCustodianshipV1 { to, at }))?;
        Ok(())
    }
}
