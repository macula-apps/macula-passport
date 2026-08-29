/// A mesh party's pubkey — the holder's own, a custodian's, or a
/// third-party requester's. One alias for all three because the wire
/// shape is identical; which role it plays is determined by which field
/// it's in, not by the type.
pub type MeshKey = Vec<u8>;
