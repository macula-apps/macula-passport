/// Whether a passport holder is a human or an animal in someone's care.
/// "Holder" because that's what a passport itself calls the person it's
/// about — not "subject", which was borrowed from an adjacent field
/// (privacy law, X.509 certs) and never actually checked against this
/// domain's own vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolderKind {
    Human,
    Animal,
}
