//! Field-extraction helpers for converting domain types to/from
//! [`Value`] — the same support-module role `macula_rust_sdk::cbor`
//! plays for wire frames, just for this crate's own event log.

use macula_rust_sdk::cbor::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecError(pub String);

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "codec error: {}", self.0)
    }
}

impl std::error::Error for CodecError {}

fn missing(key: &str) -> CodecError {
    CodecError(format!("missing field {key:?}"))
}

fn wrong_type(key: &str, expected: &str) -> CodecError {
    CodecError(format!("field {key:?} is not {expected}"))
}

pub fn text(v: &Value, key: &str) -> Result<String, CodecError> {
    match v.get(key) {
        Some(Value::Text(s)) => Ok(s.clone()),
        Some(_) => Err(wrong_type(key, "text")),
        None => Err(missing(key)),
    }
}

pub fn opt_text(v: &Value, key: &str) -> Result<Option<String>, CodecError> {
    match v.get(key) {
        Some(Value::Text(s)) => Ok(Some(s.clone())),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(wrong_type(key, "text or null")),
    }
}

pub fn int(v: &Value, key: &str) -> Result<i64, CodecError> {
    match v.get(key) {
        Some(Value::Int(i)) => i64::try_from(*i).map_err(|_| wrong_type(key, "i64-range int")),
        Some(_) => Err(wrong_type(key, "int")),
        None => Err(missing(key)),
    }
}

pub fn opt_int(v: &Value, key: &str) -> Result<Option<i64>, CodecError> {
    match v.get(key) {
        Some(Value::Int(i)) => {
            Ok(Some(i64::try_from(*i).map_err(|_| wrong_type(key, "i64-range int"))?))
        }
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(wrong_type(key, "int or null")),
    }
}

pub fn bytes(v: &Value, key: &str) -> Result<Vec<u8>, CodecError> {
    match v.get(key) {
        Some(Value::Bytes(b)) => Ok(b.clone()),
        Some(_) => Err(wrong_type(key, "bytes")),
        None => Err(missing(key)),
    }
}

pub fn opt_bytes(v: &Value, key: &str) -> Result<Option<Vec<u8>>, CodecError> {
    match v.get(key) {
        Some(Value::Bytes(b)) => Ok(Some(b.clone())),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(wrong_type(key, "bytes or null")),
    }
}

pub fn value(v: &Value, key: &str) -> Result<Value, CodecError> {
    v.get(key).cloned().ok_or_else(|| missing(key))
}

pub fn uuid(v: &Value, key: &str) -> Result<Uuid, CodecError> {
    let b = bytes(v, key)?;
    Uuid::from_slice(&b).map_err(|e| CodecError(format!("field {key:?} is not a uuid: {e}")))
}

pub fn uuid_value(id: Uuid) -> Value {
    Value::Bytes(id.as_bytes().to_vec())
}
