use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub hidden: bool,
    pub ordering: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Question {
    pub id: i64,
    pub category: Option<i64>,
    pub question: String,
    pub answer: String,
    pub attachment: Option<String>,
    pub hidden: bool,
    pub ordering: i64,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub is_admin: bool,
    pub active: bool,
}
