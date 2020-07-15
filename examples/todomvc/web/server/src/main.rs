use anyhow::Result;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenUrl,
};
use structopt::StructOpt;
use tide::{
    http::{
        cookies::{Cookie, SameSite},
        mime,
    },
    Body, Redirect, Request, Response,
};
use url::Url;

const HOME: &str = "../client/index.html";
const PKG: &str = "../client/pkg";
const PUBLIC: &str = "../client/public";

#[derive(Debug, StructOpt)]
struct Config {
    #[structopt(long, env = "API_URL")]
    api_url: Url,
    #[structopt(long, env = "REDIRECT_URL")]
    redirect_url: Url,
    #[structopt(long, parse(from_str=client_id_from_str), env = "CLIENT_ID")]
    client_id: ClientId,
    #[structopt(long, parse(from_str=client_secret_from_str), env = "CLIENT_SECRET", hide_env_values = true)]
    client_secret: ClientSecret,
    #[structopt(long, parse(try_from_str=auth_url_from_str), env = "AUTH_URL")]
    auth_url: AuthUrl,
    #[structopt(long, parse(try_from_str=token_url_from_str), env = "TOKEN_URL")]
    token_url: TokenUrl,
}

fn client_id_from_str(s: &str) -> ClientId {
    ClientId::new(s.to_string())
}

fn client_secret_from_str(s: &str) -> ClientSecret {
    ClientSecret::new(s.to_string())
}

fn auth_url_from_str(s: &str) -> Result<AuthUrl> {
    Ok(AuthUrl::new(s.to_string())?)
}

fn token_url_from_str(s: &str) -> Result<TokenUrl> {
    Ok(TokenUrl::new(s.to_string())?)
}

async fn home(req: Request<Config>) -> tide::Result {
    let mut response = Response::new(tide::StatusCode::Ok);

    let body = Body::from_file(HOME).await?;
    response.set_body(body);

    let config = req.state();
    let cookie = Cookie::build("api_url", config.api_url.to_string())
        .path("/")
        .same_site(SameSite::Strict)
        .finish();
    response.insert_cookie(cookie);

    let cookie = Cookie::build("redirect_url", config.redirect_url.to_string())
        .path("/")
        .same_site(SameSite::Strict)
        .finish();
    response.insert_cookie(cookie);

    response.set_content_type(mime::HTML);

    Ok(response)
}

async fn login(req: Request<Config>) -> tide::Result {
    let config = req.state();

    let client = BasicClient::new(
        config.client_id.clone(),
        Some(config.client_secret.clone()),
        config.auth_url.clone(),
        Some(config.token_url.clone()),
    )
    .set_redirect_url(
        RedirectUrl::new(config.redirect_url.to_string()).expect("Invalid redirect URL"),
    );

    let (pkce_code_challenge, _pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.email".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/userinfo.profile".to_string(),
        ))
        .add_scope(Scope::new("openid".to_string()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    Ok(Redirect::new(authorize_url.as_ref()).into())
}

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::start();

    let mut app = tide::with_state(Config::from_args());

    app.at("/").get(home);
    app.at("/active").get(home);
    app.at("/completed").get(home);
    app.at("/healthz").get(|_| async { Ok(Response::new(204)) });
    app.at("/login").get(login);
    app.at("/pkg").serve_dir(PKG)?;
    app.at("/public").serve_dir(PUBLIC)?;

    app.listen("0.0.0.0:8080").await?;

    Ok(())
}
