use anyhow::Result;
use structopt::StructOpt;

#[async_std::main]
async fn main() -> Result<()> {
    let config = todomvc_api::Config::from_args();

    tide::log::start();
    let app = todomvc_api::create_app(&config).await?;
    app.listen("0.0.0.0:3030").await?;

    Ok(())
}
