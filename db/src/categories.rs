use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub hidden: bool,
    pub ordering: i64,
}

pub async fn get_category(pool: &SqlitePool, id: i64) -> sqlx::Result<Category> {
    sqlx::query_as!(
        Category,
        r#"
        SELECT * FROM categories WHERE categories.id = ?1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn create_category(
    pool: &SqlitePool,
    name: &str,
    hidden: bool,
    ordering: i64,
) -> sqlx::Result<i64> {
    let mut conn = pool.acquire().await?;

    let id = sqlx::query!(
        r#"
INSERT INTO categories (name, hidden, ordering) VALUES (?1, ?2, ?3)
        "#,
        name,
        hidden,
        ordering,
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn update_category(
    pool: &SqlitePool,
    id: i64,
    name: String,
    hidden: bool,
) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        UPDATE categories SET name=?1, hidden=?2 WHERE categories.id = ?3
        "#,
        name,
        hidden,
        id,
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn get_all_categories(pool: &SqlitePool) -> sqlx::Result<Vec<Category>> {
    sqlx::query_as!(
        Category,
        r#"
        SELECT * FROM categories ORDER BY ordering
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_public_categories(pool: &SqlitePool) -> sqlx::Result<Vec<Category>> {
    sqlx::query_as!(
        Category,
        r#"
        SELECT * FROM categories WHERE hidden = FALSE ORDER BY ordering
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_category(pool: &SqlitePool, category_id: i64) -> sqlx::Result<()> {
    let mut conn = pool.acquire().await?;

    sqlx::query!(
        r#"
        DELETE FROM categories WHERE categories.id = ?1
        "#,
        category_id,
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

pub async fn reorder_categories(pool: &SqlitePool, categories: Vec<Category>) -> sqlx::Result<()> {
    let mut transaction = pool.begin().await?;
    for category in categories {
        sqlx::query!(
            r#"
            UPDATE categories SET ordering=?1 WHERE categories.id = ?2
            "#,
            category.ordering,
            category.id,
        )
        .execute(&mut transaction)
        .await?;
    }
    transaction.commit().await?;
    Ok(())
}

pub async fn import_categories(pool: &SqlitePool, categories: Vec<Category>) -> sqlx::Result<()> {
    let existing_categories = get_all_categories(pool).await?;
    let existing_categories_ids: HashSet<i64> = existing_categories.iter().map(|c| c.id).collect();
    let new_categories_ids: HashSet<i64> = categories.iter().map(|c| c.id).collect();
    for category_id in existing_categories_ids.difference(&new_categories_ids) {
        delete_category(pool, *category_id).await?;
    }
    for category in categories {
        if existing_categories_ids.contains(&category.id) {
            update_category(pool, category.id, category.name, category.hidden).await?;
        } else {
            create_category(
                pool,
                category.name.as_str(),
                category.hidden,
                category.ordering,
            )
            .await?;
        }
    }
    Ok(())
}
