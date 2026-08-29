use macula_passport::holder::HolderKind;

#[derive(uniffi::Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiHolderKind {
    Human,
    Animal,
}

impl From<FfiHolderKind> for HolderKind {
    fn from(k: FfiHolderKind) -> Self {
        match k {
            FfiHolderKind::Human => HolderKind::Human,
            FfiHolderKind::Animal => HolderKind::Animal,
        }
    }
}
