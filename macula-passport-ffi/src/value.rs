use crate::FfiError;

/// A restricted mirror of [`macula_rust_sdk::cbor::Value`] — see the
/// crate root's module doc for why `List`/`Map` are missing.
#[derive(uniffi::Enum, Debug, Clone, PartialEq)]
pub enum FfiValue {
    Null,
    Int(i64),
    Bytes(Vec<u8>),
    Text(String),
    Float(f64),
}

impl From<FfiValue> for macula_rust_sdk::cbor::Value {
    fn from(v: FfiValue) -> Self {
        use macula_rust_sdk::cbor::Value;
        match v {
            FfiValue::Null => Value::Null,
            FfiValue::Int(n) => Value::Int(n as i128),
            FfiValue::Bytes(b) => Value::Bytes(b),
            FfiValue::Text(t) => Value::Text(t),
            FfiValue::Float(f) => Value::Float(f),
        }
    }
}

impl TryFrom<macula_rust_sdk::cbor::Value> for FfiValue {
    type Error = FfiError;

    fn try_from(v: macula_rust_sdk::cbor::Value) -> Result<Self, FfiError> {
        use macula_rust_sdk::cbor::Value;
        match v {
            Value::Null => Ok(FfiValue::Null),
            Value::Int(n) => i64::try_from(n).map(FfiValue::Int).map_err(|_| {
                FfiError::UnrepresentableValue { reason: format!("integer {n} is outside i64 range") }
            }),
            Value::Bytes(b) => Ok(FfiValue::Bytes(b)),
            Value::Text(t) => Ok(FfiValue::Text(t)),
            Value::Float(f) => Ok(FfiValue::Float(f)),
            Value::List(_) => Err(FfiError::UnrepresentableValue {
                reason: "list values are not yet supported across the FFI boundary".to_string(),
            }),
            Value::Map(_) => Err(FfiError::UnrepresentableValue {
                reason: "map values are not yet supported across the FFI boundary".to_string(),
            }),
        }
    }
}
