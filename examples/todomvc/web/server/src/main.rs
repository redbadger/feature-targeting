use anyhow::Result;

const HOME: &str = "../client/index.html";
const PKG: &str = "../client/pkg";
const PUBLIC: &str = "../client/public";

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::start();

    let mut app = tide::new();
    let home = |_| async { Ok(tide::Body::from_file(HOME).await?) };

    app.at("/").get(home);
    app.at("/active").get(home);
    app.at("/completed").get(home);
    app.at("/pkg").serve_dir(PKG)?;
    app.at("/public").serve_dir(PUBLIC)?;

    app.listen("0.0.0.0:8080").await?;

    Ok(())
}
