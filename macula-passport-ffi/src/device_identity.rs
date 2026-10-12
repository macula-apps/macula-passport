//! The device's own mesh identity: the key a holder acts as on their
//! phone (`acting_as` on every claim desk). Kept as an owner-only key
//! file in a directory the app names, normally inside its private
//! storage, so it never leaves the phone. Created once, with the
//! admission puzzle solved, on the fleet's `pq_hybrid` profile; every
//! later start loads the same key. Nothing here touches the mesh.

use std::path::Path;

use macula_rust::node_key::{NodeKey, DEFAULT_IDENTITY_NAME};
use macula_rust::profile::Profile;

use crate::FfiError;

/// The carried public key of this device's identity in `identity_dir`,
/// created there on the first call. The directory is created when it
/// does not exist. A key file there that does not load is refused,
/// never replaced.
#[uniffi::export]
pub fn device_mesh_key(identity_dir: String) -> Result<Vec<u8>, FfiError> {
    let dir = Path::new(&identity_dir);
    std::fs::create_dir_all(dir).map_err(|e| FfiError::Identity {
        reason: e.to_string(),
    })?;
    NodeKey::stored_identity(dir, DEFAULT_IDENTITY_NAME, Profile::PqHybrid)
        .map(|(key, _path)| key.public_key())
        .map_err(|e| FfiError::Identity {
            reason: e.to_string(),
        })
}
