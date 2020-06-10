use anyhow::Result;
use sqlx::{postgres::PgRow, FromRow, PgPool, Row};

#[derive(FromRow, Clone)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub completed: bool,
    pub order: Option<i32>,
}

impl Todo {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Todo>> {
        let mut todos = vec![];
        let records = sqlx::query!(
            r#"
                SELECT id, title, completed, item_order
                    FROM todos
                ORDER BY id
            "#
        )
        .fetch_all(pool)
        .await?;

        for rec in records {
            todos.push(Todo {
                id: rec.id,
                title: rec.title,
                completed: rec.completed,
                order: rec.item_order,
            });
        }

        Ok(todos)
    }

    pub async fn find_by_id(id: i32, pool: &PgPool) -> Result<Todo> {
        let rec = sqlx::query!(
            r#"
                    SELECT * FROM todos WHERE id = $1
                "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(Todo {
            id: rec.id,
            title: rec.title,
            completed: rec.completed,
            order: rec.item_order,
        })
    }

    pub async fn create(title: String, order: Option<i32>, pool: &PgPool) -> Result<Todo> {
        let mut tx = pool.begin().await?;
        let todo = sqlx::query(
            "INSERT INTO todos (title, item_order) VALUES ($1, $2) RETURNING id, title, completed, item_order",
        )
        .bind(&title)
        .bind(order)
        .map(|row: PgRow| Todo {
            id: row.get(0),
            title: row.get(1),
            completed: row.get(2),
            order: row.get(3),
        })
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(todo)
    }

    pub async fn update(
        id: i32,
        title: String,
        completed: bool,
        order: Option<i32>,
        pool: &PgPool,
    ) -> Result<Todo> {
        let mut tx = pool.begin().await.unwrap();
        let todo = sqlx::query("UPDATE todos SET title = $1, completed = $2, order = $3 WHERE id = $4 RETURNING id, title, completed, item_order")
            .bind(&title)
            .bind(completed)
            .bind(order)
            .bind(id)
            .map(|row: PgRow| {
                Todo {
                    id: row.get(0),
                    title: row.get(1),
                    completed: row.get(2),
                    order: row.get(3),
                }
            })
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await.unwrap();
        Ok(todo)
    }

    pub async fn delete(id: i32, pool: &PgPool) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let deleted = sqlx::query("DELETE FROM todos WHERE id = $1")
            .bind(id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(deleted)
    }
}
