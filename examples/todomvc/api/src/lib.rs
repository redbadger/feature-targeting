use anyhow::Result;
use http_types::headers::HeaderValue;
use sqlx::postgres::PgPoolOptions;
use tide::{
    security::{CorsMiddleware, Origin},
    Response, Server,
};

mod db;
mod graphql;

pub async fn create_app(database_url: &str) -> Result<Server<graphql::State>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    let mut app = tide::with_state(graphql::State::new(pool));

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

    app.at("/healthz").get(|_| async { Ok(Response::new(204)) });
    app.at("/").post(graphql::handle_graphql);
    app.at("/").get(graphql::handle_graphiql);

    Ok(app)
}
