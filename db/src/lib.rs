pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
#[cfg(test)]
extern crate diesel_migrations;

extern crate dotenv;

use crate::models::{Category, NewCategory, NewQuestion, Question};

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> ConnectionResult<SqliteConnection> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
}

//todo: return Result
pub fn create_category(conn: &SqliteConnection, name: &str) {
    use schema::categories;
    let new_category = NewCategory { name };
    diesel::insert_into(categories::table)
        .values(&new_category)
        .execute(conn)
        .unwrap();
}

pub fn get_categories(conn: &SqliteConnection) -> Vec<Category> {
    use crate::schema::categories::dsl::*;
    categories
        .load::<Category>(conn)
        .expect("Error loading posts")
}

pub fn delete_questions(conn: &SqliteConnection) {
    use schema::questions;
    diesel::delete(questions::table).execute(conn).unwrap();
}

pub fn get_question_by_id(conn: &SqliteConnection, id_: i32) -> Question {
    use crate::schema::questions::dsl::*;
    questions
        .filter(id.eq(id_))
        .first::<Question>(conn)
        .expect("Error loading posts")
}

//todo: return Result
pub fn create_question(
    conn: &SqliteConnection,
    question: &str,
    answer: &str,
    category: Option<i32>,
) {
    use schema::questions;
    let new_question = NewQuestion {
        category,
        question,
        answer,
    };
    diesel::insert_into(questions::table)
        .values(&new_question)
        .execute(conn)
        .unwrap();
}

pub fn get_questions_by_category(conn: &SqliteConnection, category_id: i32) -> Vec<Question> {
    use crate::schema::questions::dsl::*;
    questions
        .filter(category.eq(category_id))
        .load::<Question>(conn)
        .expect("Error loading posts")
}

pub fn delete_categories(conn: &SqliteConnection) {
    use schema::categories;
    diesel::delete(categories::table).execute(conn).unwrap();
}

#[cfg(test)]
mod tests {
    extern crate diesel;
    use super::models::*;
    use super::*;

    #[test]
    #[ignore]
    fn test_categories_crud() {
        use crate::schema::categories::dsl::*;
        let connection = establish_connection().unwrap();
        create_category(&connection, "test");
        let results = categories
            .limit(5)
            .load::<Category>(&connection)
            .expect("Error loading posts");
        assert!(!results.is_empty());
        delete_categories(&connection);
    }

    #[test]
    #[ignore]
    fn test_questions_crud() {
        use crate::schema::questions::dsl::*;
        let connection = establish_connection().unwrap();
        create_question(&connection, "test", "test", None);
        let results = questions
            .limit(5)
            .load::<Question>(&connection)
            .expect("Error loading posts");
        assert!(!results.is_empty());
        delete_questions(&connection);
    }
}
