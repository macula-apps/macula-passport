use macula_passport::grant::Grant;

/// A grant's `id` crosses as raw bytes, same as everywhere else in this
/// crate — but a caller can never *construct* one: the only way to
/// obtain a grant id is a prior `desks::grant_data_access` return value,
/// since `Grant::id` is always freshly minted inside `macula_passport`,
/// never accepted as input.
#[derive(uniffi::Record, Debug, Clone)]
pub struct FfiGrant {
    pub id: Vec<u8>,
    pub claim_type_prefix: String,
    pub requester: Vec<u8>,
    pub purpose: String,
    pub expires_at: Option<i64>,
}

impl From<Grant> for FfiGrant {
    fn from(g: Grant) -> Self {
        FfiGrant {
            id: g.id.as_bytes().to_vec(),
            claim_type_prefix: g.claim_type_prefix,
            requester: g.requester,
            purpose: g.purpose,
            expires_at: g.expires_at,
        }
    }
}
