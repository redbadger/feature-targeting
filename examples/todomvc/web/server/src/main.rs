use anyhow::{Context, Result};
use tide::{
    http::{
        cookies::{Cookie, SameSite},
        mime,
    },
    Body, Request, Response,
};

const HOME: &str = "../client/index.html";
const PKG: &str = "../client/pkg";
const PUBLIC: &str = "../client/public";

#[async_std::main]
async fn main() -> Result<()> {
    let api_url = std::env::var("API_URL").context("API_URL env var not found")?;

    tide::log::start();

    let mut app = tide::with_state(api_url);

    app.at("/").get(home);
    app.at("/active").get(home);
    app.at("/completed").get(home);
    app.at("/pkg").serve_dir(PKG)?;
    app.at("/public").serve_dir(PUBLIC)?;
    app.at("/healthz").get(|_| async { Ok(Response::new(204)) });

    app.listen("0.0.0.0:8080").await?;

    Ok(())
}

async fn home(req: Request<String>) -> tide::Result {
    let mut response = Response::new(tide::StatusCode::Ok);

    let body = Body::from_file(HOME).await?;
    response.set_body(body);

    let api_url = req.state().clone();
    let cookie = Cookie::build("api_url", api_url)
        .path("/")
        .same_site(SameSite::Strict)
        .finish();
    response.insert_cookie(cookie);

    response.set_content_type(mime::HTML);

    Ok(response)
}
