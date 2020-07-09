use anyhow::Result;
use http_types::headers::HeaderValue;
use sqlx::PgPool;
use tide::{
    security::{CorsMiddleware, Origin},
    Redirect, Response, Server,
};

mod db;
mod graphql;

pub async fn create_app(database_url: &str) -> Result<Server<graphql::State>> {
    let connection_pool = PgPool::new(database_url).await?;

    let mut app = tide::with_state(graphql::State::new(connection_pool));

    app.middleware(
        CorsMiddleware::new()
            .allow_methods(
                "GET, POST, OPTIONS"
                    .parse::<HeaderValue>()
                    .expect("could not parse as HTTP header value"),
            )
            .allow_origin(Origin::from("*"))
            .allow_credentials(false),
    );

    app.at("/").get(Redirect::permanent("/graphiql"));
    app.at("/healthz").get(|_| async { Ok(Response::new(204)) });
    app.at("/graphql").post(graphql::handle_graphql);
    app.at("/graphiql").get(graphql::handle_graphiql);

    Ok(app)
}
