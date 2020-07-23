use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
struct Config {
    #[structopt(long, env = "DATABASE_URL")]
    database_url: String,
}

#[async_std::main]
async fn main() -> Result<()> {
    let config = Config::from_args();

    tide::log::start();
    let app = todomvc_api::create_app(&config.database_url).await?;
    app.listen("0.0.0.0:3030").await?;

    Ok(())
}
