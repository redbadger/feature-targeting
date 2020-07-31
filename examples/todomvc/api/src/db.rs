#![allow(clippy::suspicious_else_formatting, clippy::toplevel_ref_arg)] // try removing this when sqlx is updated
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
    pub async fn find_all(pool: &PgPool, auth_subject: &str) -> Result<Vec<Todo>> {
        let todos = sqlx::query_file_as!(Todo, "sql/find_all.sql", auth_subject)
            .fetch_all(pool)
            .await?;

        Ok(todos)
    }

    pub async fn find_by_id(pool: &PgPool, auth_subject: &str, id: Uuid) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/find_by_id.sql", id, auth_subject)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }

    pub async fn create(pool: &PgPool, auth_subject: &str, title: String) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/create.sql", auth_subject, title)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }

    pub async fn update(
        pool: &PgPool,
        auth_subject: &str,
        id: Uuid,
        title: Option<String>,
        completed: Option<bool>,
    ) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/update.sql", id, auth_subject, title, completed)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }

    pub async fn delete(pool: &PgPool, auth_subject: &str, id: Uuid) -> Result<Todo> {
        let todo = sqlx::query_file_as!(Todo, "sql/delete.sql", id, auth_subject)
            .fetch_one(pool)
            .await?;

        Ok(todo)
    }
}
