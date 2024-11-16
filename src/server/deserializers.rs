use serde::{Deserialize, Deserializer};

// forms send "on" or not including the value for checkbox
pub fn deserialize_bool_from_checkbox<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    if let Some(value) = value {
        match value.as_str() {
            "on" => Ok(Some(true)),
            variant => Err(serde::de::Error::unknown_variant(variant, &["on"])),
        }
    } else {
        Ok(None)
    }
}

// a hack to deserialize array of strings to Vec<i64>, since this is how it's get encoded and
// I don't want to write JS to do it on frontend
#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct Stri64(pub i64);

impl TryFrom<String> for Stri64 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.parse::<i64>() {
            Ok(v) => Ok(Stri64(v)),
            Err(_) => Err(format!("Wrong value {value}, can not parse to i64")),
        }
    }
}
