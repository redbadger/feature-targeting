use anyhow::Result;
use dotenv::dotenv;
use std::env;

#[async_std::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL")?;
    let app = todomvc_api::create_app(&database_url).await?;
    app.listen("0.0.0.0:3030").await?;

    Ok(())
}
