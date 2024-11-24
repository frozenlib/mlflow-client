use serde_json::Value;

use crate::data::Param;
use crate::{Error, Result};

pub(crate) fn none_if_not_exist<T, U>(
    value: Result<T>,
    f: impl FnOnce(T) -> Result<U>,
) -> Result<Option<U>> {
    match value {
        Ok(value) => Ok(Some(f(value)?)),
        Err(e) if e.is_resource_does_not_exist() => Ok(None),
        Err(e) => Err(e),
    }
}
pub(crate) fn build_params(key: &str, value: &Value, params: &mut Vec<Param>) -> Result<()> {
    match value {
        Value::Null => {}
        Value::Bool(value) => params.push(Param {
            key: key.to_string(),
            value: value.to_string(),
        }),
        Value::Number(value) => params.push(Param {
            key: key.to_string(),
            value: value.to_string(),
        }),
        Value::String(value) => params.push(Param {
            key: key.to_string(),
            value: value.to_string(),
        }),
        Value::Array(_) => {
            return Err(Error::from_message("Array not supported"));
        }
        Value::Object(m) => {
            let s = if key.is_empty() { "" } else { "." };
            for (k, v) in m {
                build_params(&format!("{key}{s}{k}"), v, params)?;
            }
        }
    }
    Ok(())
}
