use anyhow::Result;
use http_types::headers::HeaderValue;
use sqlx::postgres::PgPoolOptions;
use structopt::StructOpt;
use tide::{
    security::{CorsMiddleware, Origin},
    Response, Server,
};

mod db;
mod graphql;
mod jwt;

#[derive(Debug, Clone, StructOpt)]
pub struct Config {
    #[structopt(long, env = "DATABASE_URL")]
    database_url: String,
    #[structopt(long, env = "MOUNTED_AT", default_value = "/")]
    mounted_at: String,
}

pub async fn create_app(config: &Config) -> Result<Server<graphql::State>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    let mut app = tide::with_state(graphql::State::new(pool, &config.mounted_at));

    app.with(
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
