pub struct Category {
    pub id: i64,
    pub name: String,
}

pub struct Question {
    pub id: i64,
    pub category: Option<i64>,
    pub question: String,
    pub answer: String,
}

pub struct User {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: String,
}
