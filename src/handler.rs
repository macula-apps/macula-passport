//! Routes a [`Command`] to its own desk module's handler and wraps the
//! result as a [`PassportEvent`] — no business logic lives here, see
//! the modules under [`crate::desks`] for that.

use crate::desks::disclose_data::DiscloseDataOutcome;
use crate::desks::{
    assign_custodian, capture_biometric_sample, disclose_data, grant_data_access,
    initiate_passport, record_health_observation, register_identity_document, revoke_data_access,
    transfer_custodianship,
};
use crate::dossier::{Command, Dossier, PassportEvent};
use crate::error::DomainError;

pub fn handle(state: &Dossier, cmd: Command) -> Result<PassportEvent, DomainError> {
    Ok(match cmd {
        Command::InitiatePassportV1(c) => PassportEvent::PassportInitiatedV1(
            initiate_passport::maybe_initiate_passport(state, c)?,
        ),
        Command::AssignCustodianV1(c) => {
            PassportEvent::CustodianAssignedV1(assign_custodian::maybe_assign_custodian(state, c)?)
        }
        Command::TransferCustodianshipV1(c) => PassportEvent::CustodianshipTransferredV1(
            transfer_custodianship::maybe_transfer_custodianship(state, c)?,
        ),
        Command::RegisterIdentityDocumentV1(c) => PassportEvent::IdentityDocumentRegisteredV1(
            register_identity_document::maybe_register_identity_document(state, c)?,
        ),
        Command::CaptureBiometricSampleV1(c) => PassportEvent::BiometricSampleCapturedV1(
            capture_biometric_sample::maybe_capture_biometric_sample(state, c)?,
        ),
        Command::RecordHealthObservationV1(c) => PassportEvent::HealthObservationRecordedV1(
            record_health_observation::maybe_record_health_observation(state, c)?,
        ),
        Command::GrantDataAccessV1(c) => PassportEvent::DataAccessGrantedV1(
            grant_data_access::maybe_grant_data_access(state, c)?,
        ),
        Command::RevokeDataAccessV1(c) => PassportEvent::DataAccessRevokedV1(
            revoke_data_access::maybe_revoke_data_access(state, c)?,
        ),
        Command::DiscloseDataV1(c) => match disclose_data::decide_disclosure(state, c) {
            DiscloseDataOutcome::Disclosed(e) => PassportEvent::DataDisclosedV1(e),
            DiscloseDataOutcome::Denied(e) => PassportEvent::DataAccessDeniedV1(e),
        },
    })
}
