use anyhow::Result;
use dotenv::dotenv;
use std::env;

fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    smol::run(start(
        &env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file"),
    ))
}

async fn start(database_url: &str) -> Result<()> {
    let app = todomvc_api::create_app(database_url).await?;
    app.listen("0.0.0.0:3030").await?;
    Ok(())
}
