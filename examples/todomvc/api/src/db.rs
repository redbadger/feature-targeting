use anyhow::Result;
use sqlx::{types::Uuid, PgPool};

#[derive(Clone)]
pub struct Todo {
    pub id: Uuid,
    pub auth_subject: String,
    pub title: String,
    pub completed: bool,
}

impl Todo {
    pub async fn find_all(auth_subject: &str, pool: &PgPool) -> Result<Vec<Todo>> {
        let todos = sqlx::query_file_as!(Todo, "sql/find_all.sql", auth_subject)
            .fetch_all(pool)
            .await?;

        Ok(todos)
    }

    pub async fn find_by_id(id: Uuid, auth_subject: &str, pool: &PgPool) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/find_by_id.sql", id, auth_subject)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }

    pub async fn create(auth_subject: &str, title: String, pool: &PgPool) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/create.sql", auth_subject, title)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }

    pub async fn update(
        id: Uuid,
        auth_subject: &str,
        title: Option<String>,
        completed: Option<bool>,
        pool: &PgPool,
    ) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/update.sql", id, auth_subject, title, completed)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }

    pub async fn delete(id: Uuid, auth_subject: &str, pool: &PgPool) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/delete.sql", id, auth_subject)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }
}
