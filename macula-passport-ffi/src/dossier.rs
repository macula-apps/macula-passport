use std::path::Path;
use std::sync::Mutex;

use macula_passport::dossier::{Command, Dossier, PassportEvent};
use macula_passport::store::Store;
use uuid::Uuid;

use crate::biometric_sample::FfiBiometricSample;
use crate::claim::FfiClaim;
use crate::grant::FfiGrant;
use crate::{to_uuid, FfiError};

/// One holder's dossier, backed by a SQLite file at `path`. Every
/// method exported on this type — one `impl FfiPassport` block per
/// desk under `crate::desks`, plus the read-only queries below, which
/// aren't any single desk's concern the same way `Dossier`'s own query
/// methods in `macula_passport::dossier` live on the aggregate, not on
/// a desk — does
/// load→replay→decide→append (or load→replay→read) as one call; the
/// mobile side never manually replays.
#[derive(uniffi::Object)]
pub struct FfiPassport {
    store: Mutex<Store>,
    holder_id: Uuid,
}

impl FfiPassport {
    pub(crate) fn apply(&self, cmd: Command) -> Result<PassportEvent, FfiError> {
        let mut store = self.store.lock().expect("store mutex poisoned");
        let events = store.load(self.holder_id)?;
        let state = Dossier::replay(&events);
        let event = macula_passport::handler::handle(&state, cmd)?;
        store.append(self.holder_id, &event)?;
        Ok(event)
    }

    pub(crate) fn state(&self) -> Result<Dossier, FfiError> {
        let store = self.store.lock().expect("store mutex poisoned");
        let events = store.load(self.holder_id)?;
        Ok(Dossier::replay(&events))
    }
}

#[uniffi::export]
impl FfiPassport {
    #[uniffi::constructor]
    pub fn open(path: String, holder_id: Vec<u8>) -> Result<Self, FfiError> {
        Ok(FfiPassport {
            store: Mutex::new(Store::open(Path::new(&path))?),
            holder_id: to_uuid(holder_id)?,
        })
    }

    pub fn is_initiated(&self) -> Result<bool, FfiError> {
        Ok(self.state()?.is_initiated())
    }

    pub fn custodian(&self) -> Result<Option<Vec<u8>>, FfiError> {
        Ok(self.state()?.custodian)
    }

    pub fn list_claims(&self) -> Result<Vec<FfiClaim>, FfiError> {
        self.state()?
            .claims
            .into_iter()
            .map(FfiClaim::try_from)
            .collect()
    }

    pub fn list_biometric_samples(&self) -> Result<Vec<FfiBiometricSample>, FfiError> {
        Ok(self
            .state()?
            .biometric_samples
            .into_iter()
            .map(FfiBiometricSample::from)
            .collect())
    }

    pub fn list_active_grants(&self) -> Result<Vec<FfiGrant>, FfiError> {
        let state = self.state()?;
        Ok(state.active_grants().cloned().map(FfiGrant::from).collect())
    }
}
