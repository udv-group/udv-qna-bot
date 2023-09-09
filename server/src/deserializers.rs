use serde::{Deserialize, Deserializer};

// forms send "on" or not including the value for checkbox
pub fn deserialize_bool_from_checkbox<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    if value.is_none() {
        return Ok(None);
    } else {
        match value.unwrap().as_str() {
            "on" => Ok(Some(true)),
            variant => Err(serde::de::Error::unknown_variant(variant, &["on"])),
        }
    }
}
