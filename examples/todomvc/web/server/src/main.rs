use anyhow::{anyhow, Result};
use failure::Fail;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    AccessTokenHash, AsyncCodeTokenRequest, AuthUrl, AuthenticationFlow, AuthorizationCode,
    ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope,
};
use serde::Deserialize;
use structopt::StructOpt;
use tide::{
    http::{
        cookies::{Cookie, SameSite},
        mime,
    },
    Body, Redirect, Request, Response,
};
use url::Url;

mod async_client;

const HOME: &str = "../client/index.html";
const PKG: &str = "../client/pkg";
const PUBLIC: &str = "../client/public";

#[derive(Debug, Clone, StructOpt)]
struct Config {
    #[structopt(long, env = "API_URL")]
    api_url: Url,
    #[structopt(long, parse(try_from_str=redirect_url_from_str), env = "REDIRECT_URL")]
    redirect_url: RedirectUrl,
    #[structopt(long, parse(from_str=client_id_from_str), env = "CLIENT_ID")]
    client_id: ClientId,
    #[structopt(long, parse(from_str=client_secret_from_str), env = "CLIENT_SECRET", hide_env_values = true)]
    client_secret: ClientSecret,
    #[structopt(long, parse(try_from_str=auth_url_from_str), env = "AUTH_URL")]
    auth_url: AuthUrl,
}

fn redirect_url_from_str(s: &str) -> Result<RedirectUrl> {
    Ok(RedirectUrl::new(s.to_string())?)
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

#[derive(Clone)]
struct State {
    client: CoreClient,
    config: Config,
}

async fn home(req: Request<State>) -> tide::Result {
    let mut res = Response::new(tide::StatusCode::Ok);

    let body = Body::from_file(HOME).await?;
    res.set_body(body);

    let config = req.state().config.clone();
    let cookie = Cookie::build("api_url", config.api_url.to_string())
        .path("/")
        .same_site(SameSite::Strict)
        .finish();
    res.insert_cookie(cookie);

    let cookie = Cookie::build("redirect_url", config.redirect_url.to_string())
        .path("/")
        .same_site(SameSite::Strict)
        .finish();
    res.insert_cookie(cookie);

    res.set_content_type(mime::HTML);

    Ok(res)
}

async fn login(req: Request<State>) -> tide::Result {
    let state = req.state();
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state, nonce) = state
        .client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    let mut res: Response = Redirect::new(authorize_url.as_ref()).into();

    let cookie = Cookie::build("csrf_state", csrf_state.secret().clone())
        .path("/")
        // .same_site(SameSite::Strict)
        // .http_only(true)
        // .secure(true)
        .finish();
    res.insert_cookie(cookie);

    let cookie = Cookie::build("pkce_code_verifier", pkce_code_verifier.secret().clone())
        .path("/")
        // .same_site(SameSite::Strict)
        // .http_only(true)
        // .secure(true)
        .finish();
    res.insert_cookie(cookie);

    let cookie = Cookie::build("nonce", nonce.secret().clone())
        .path("/")
        // .same_site(SameSite::Strict)
        // .http_only(true)
        // .secure(true)
        .finish();
    res.insert_cookie(cookie);

    Ok(res)
}

async fn callback(req: Request<State>) -> tide::Result {
    let query: CallbackQuery = req.query()?;
    let state = req.state();
    if let Some(csrf_state_cookie) = req.cookie("csrf_state") {
        if query.state == csrf_state_cookie.value() {
            if let Some(pkce_code_verifier) = req.cookie("pkce_code_verifier") {
                let pkce_code_verifier =
                    PkceCodeVerifier::new(pkce_code_verifier.value().to_string());
                if let Some(nonce) = req.cookie("nonce") {
                    let nonce = Nonce::new(nonce.value().to_string());
                    if let Ok(token_response) = state
                        .client
                        .exchange_code(AuthorizationCode::new(query.code))
                        .set_pkce_verifier(pkce_code_verifier)
                        .request_async(async_client::async_http_client)
                        .await
                    {
                        let id_token_verifier = state.client.id_token_verifier();
                        tide::log::info!("token: {:?}", token_response);
                        if let Some(id_token) = token_response.extra_fields().id_token() {
                            let claims =
                                id_token.claims(&id_token_verifier, &nonce).map_err(|e| {
                                    tide::Error::new(
                                        tide::http::StatusCode::Unauthorized,
                                        e.compat(),
                                    )
                                })?;
                            if let Some(expected_access_token_hash) = claims.access_token_hash() {
                                let actual_access_token_hash = AccessTokenHash::from_token(
                                    token_response.access_token(),
                                    &id_token.signing_alg().map_err(|e| {
                                        tide::Error::new(
                                            tide::http::StatusCode::Unauthorized,
                                            e.compat(),
                                        )
                                    })?,
                                )
                                .map_err(|e| {
                                    tide::Error::new(
                                        tide::http::StatusCode::Unauthorized,
                                        e.compat(),
                                    )
                                })?;
                                if actual_access_token_hash != *expected_access_token_hash {
                                    return Ok(Response::new(tide::StatusCode::Unauthorized));
                                }
                            }
                            let mut res: Response = Redirect::new("/").into();
                            let token = id_token.to_string();
                            let cookie = Cookie::build("token", token)
                                .path("/")
                                // .same_site(SameSite::Strict)
                                // .http_only(true)
                                // .secure(true)
                                .finish();
                            res.insert_cookie(cookie);
                            return Ok(res);
                        }
                    }
                }
            }
        }
    }
    Ok(Response::new(tide::StatusCode::Unauthorized))
}

#[derive(Deserialize)]
struct CallbackQuery {
    state: String,
    code: String,
}

#[async_std::main]
async fn main() -> Result<()> {
    tide::log::start();

    let config = Config::from_args();

    let issuer_url =
        IssuerUrl::new("https://accounts.google.com".to_string()).expect("Invalid issuer URL");
    let provider_metadata =
        CoreProviderMetadata::discover_async(issuer_url, async_client::async_http_client)
            .await
            .map_err(|e| anyhow!("Could not get provider metadata: {:?}", e))?;
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        config.client_id.clone(),
        Some(config.client_secret.clone()),
    )
    .set_redirect_uri(config.redirect_url.clone());

    let mut app = tide::with_state(State { client, config });

    app.at("/").get(home);
    app.at("/active").get(home);
    app.at("/completed").get(home);
    app.at("/healthz").get(|_| async { Ok(Response::new(204)) });
    app.at("/login").get(login);
    app.at("/callback").get(callback);
    app.at("/pkg").serve_dir(PKG)?;
    app.at("/public").serve_dir(PUBLIC)?;

    app.listen("0.0.0.0:8080").await?;

    Ok(())
}
