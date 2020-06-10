use anyhow::Result;
use sqlx::PgPool;

#[derive(Clone)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub completed: bool,
    pub order: Option<i32>,
}

impl Todo {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Todo>> {
        let records = sqlx::query_file_as!(Todo, "sql/find_all.sql",)
            .fetch_all(pool)
            .await?;

        Ok(records)
    }

    pub async fn find_by_id(id: i32, pool: &PgPool) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/find_by_id.sql", id,)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }

    pub async fn create(title: String, order: Option<i32>, pool: &PgPool) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/create.sql", title, order,)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }

    pub async fn update(
        id: i32,
        title: Option<String>,
        completed: Option<bool>,
        order: Option<i32>,
        pool: &PgPool,
    ) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/update.sql", title, completed, order, id)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }

    pub async fn delete(id: i32, pool: &PgPool) -> Result<u64> {
        let deleted = sqlx::query_file!("sql/delete.sql", id)
            .execute(pool)
            .await?;

        Ok(deleted)
    }
}
