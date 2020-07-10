use anyhow::{Context, Result};
use dotenv::dotenv;
use std::env;

#[async_std::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").context("API_URL env var not found")?;

    tide::log::start();
    let app = todomvc_api::create_app(&database_url).await?;
    app.listen("0.0.0.0:3030").await?;

    Ok(())
}
