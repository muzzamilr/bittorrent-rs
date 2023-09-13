use super::result::Result;
use serde_bencode::{self, value::Value};

pub fn decode(encoded_value: &str) -> Result<Value> {
    Ok(serde_bencode::from_str::<Value>(encoded_value)?)
}

pub trait ValueToString {
    fn to_string(&self) -> Result<String>;
}

impl ValueToString for Value {
    fn to_string(&self) -> Result<String> {
        return match self {
            Value::Int(i) => Ok(i.to_string()),
            Value::List(l) => Ok(format!(
                "[{}]",
                l.iter()
                    .map(|v| v.to_string())
                    .collect::<Result<Vec<String>>>()?
                    .join(",")
            )),
            Value::Bytes(b) => Ok(format!("{:?}", String::from_utf8(b.clone())?)),
            Value::Dict(d) => {
                let mut result: Vec<String> = Vec::new();
                for (key, value) in d {
                    let key_str = String::from_utf8_lossy(key).to_string();
                    result.push(format!("\"{}\":{}", key_str, value.to_string()?));
                }
                result.sort();
                Ok(format!("{{{}}}", result.join(",")))
            }
        };
    }
}
