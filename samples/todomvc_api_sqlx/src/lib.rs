use anyhow::Result;
use sqlx::PgPool;
use tide::{Redirect, Server};

mod db;
pub mod graphql;

pub async fn create_app(database_url: &str) -> Result<Server<graphql::State>> {
    let db_pool = PgPool::new(database_url).await?;

    let mut app = Server::with_state(graphql::State { todos: db_pool });
    app.at("/").get(Redirect::permanent("/graphiql"));
    app.at("/graphql").post(graphql::handle_graphql);
    app.at("/graphiql").get(graphql::handle_graphiql);
    Ok(app)
}
