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
        let records = sqlx::query_as!(
            Todo,
            r#"
                SELECT id,
                    title,
                    completed,
                    item_order as order
                FROM todos
                ORDER BY id
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(records)
    }

    pub async fn find_by_id(id: i32, pool: &PgPool) -> Result<Todo> {
        let record = sqlx::query_as!(
            Todo,
            r#"
                SELECT id,
                    title,
                    completed,
                    item_order as order
                FROM todos
                WHERE id = $1        
            "#,
            id,
        )
        .fetch_one(pool)
        .await?;

        Ok(record)
    }

    pub async fn create(title: String, order: Option<i32>, pool: &PgPool) -> Result<Todo> {
        let mut tx = pool.begin().await?;
        let todo = sqlx::query_as!(
            Todo,
            r#"
                INSERT INTO todos (title, item_order)
                VALUES ($1, $2)
                RETURNING id,
                    title,
                    completed,
                    item_order AS order
            "#,
            &title,
            order,
        )
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
        let todo = sqlx::query_as!(
            Todo,
            r#"
                UPDATE todos
                SET title = $1,
                    completed = $2,
                    item_order = $3
                WHERE id = $4
                RETURNING id,
                    title,
                    completed,
                    item_order AS order
            "#,
            &title,
            completed,
            order,
            id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await.unwrap();
        Ok(todo)
    }

    pub async fn delete(id: i32, pool: &PgPool) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let deleted = sqlx::query!(
            r#"
                DELETE FROM todos
                WHERE id = $1
            "#,
            id
        )
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(deleted)
    }
}
