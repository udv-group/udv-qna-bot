mod categories;
mod questions;
mod users;

pub use categories::category_router;
pub use questions::questions_router;
use serde::Deserialize;
pub use users::users_router;

// a hack to deserialize array of strings to Vec<i64>, since this is how it's get encoded and
// I don't want to write JS to do it on frontend
#[derive(Deserialize)]
#[serde(try_from = "String")]
struct Stri64(pub i64);

impl TryFrom<String> for Stri64 {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.parse::<i64>() {
            Ok(v) => Ok(Stri64(v)),
            Err(_) => Err(format!("Wrong value {value}, can not parse to i64")),
        }
    }
}
