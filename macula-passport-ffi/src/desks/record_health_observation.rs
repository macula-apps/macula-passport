use macula_passport::desks::record_health_observation::RecordHealthObservationV1;
use macula_passport::dossier::Command;

use crate::claim::FfiClaim;
use crate::dossier::FfiPassport;
use crate::FfiError;

#[uniffi::export]
impl FfiPassport {
    pub fn record_health_observation(
        &self,
        claim: FfiClaim,
        acting_as: Vec<u8>,
    ) -> Result<(), FfiError> {
        self.apply(Command::RecordHealthObservationV1(RecordHealthObservationV1 {
            claim: claim.into(),
            acting_as,
        }))?;
        Ok(())
    }
}
