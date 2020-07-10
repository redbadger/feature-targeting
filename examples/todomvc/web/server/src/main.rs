use anyhow::{Context, Result};
use tide::{
    http::{
        cookies::{Cookie, SameSite},
        mime,
    },
    Body, Request, Response,
};
use url::Url;

const HOME: &str = "../client/index.html";
const PKG: &str = "../client/pkg";
const PUBLIC: &str = "../client/public";

struct State {
    api_url: Url,
    redirect_url: Url,
}

impl State {
    fn new(api_url: Url, redirect_url: Url) -> Self {
        Self {
            api_url,
            redirect_url,
        }
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::start();

    let api_url = Url::parse(&std::env::var("API_URL").context("API_URL env var not found")?)
        .expect("can't parse API_URL");
    let redirect_url =
        Url::parse(&std::env::var("REDIRECT_URL").context("REDIRECT_URL env var not found")?)
            .expect("can't parse REDIRECT_URL");

    let mut app = tide::with_state(State::new(api_url, redirect_url));

    app.at("/").get(home);
    app.at("/active").get(home);
    app.at("/completed").get(home);
    app.at("/pkg").serve_dir(PKG)?;
    app.at("/public").serve_dir(PUBLIC)?;
    app.at("/healthz").get(|_| async { Ok(Response::new(204)) });

    app.listen("0.0.0.0:8080").await?;

    Ok(())
}

async fn home(req: Request<State>) -> tide::Result {
    let mut response = Response::new(tide::StatusCode::Ok);

    let body = Body::from_file(HOME).await?;
    response.set_body(body);

    let state = req.state();
    let cookie = Cookie::build("api_url", state.api_url.to_string())
        .path("/")
        .same_site(SameSite::Strict)
        .finish();
    response.insert_cookie(cookie);

    let cookie = Cookie::build("redirect_url", state.redirect_url.to_string())
        .path("/")
        .same_site(SameSite::Strict)
        .finish();
    response.insert_cookie(cookie);

    response.set_content_type(mime::HTML);

    Ok(response)
}
