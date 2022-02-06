#[derive(Queryable)]
pub struct Category {
    pub id: i32,
    pub name: String,
}
#[derive(Queryable)]
pub struct Question {
    pub id: i32,
    pub category: Option<i32>,
    pub question: String,
    pub answer: String,
}

use super::schema::categories;
use super::schema::questions;

#[derive(Insertable)]
#[table_name = "questions"]
pub struct NewQuestion<'a> {
    pub category: Option<i32>,
    pub question: &'a str,
    pub answer: &'a str,
}

#[derive(Insertable)]
#[table_name = "categories"]
pub struct NewCategory<'a> {
    pub name: &'a str,
}
