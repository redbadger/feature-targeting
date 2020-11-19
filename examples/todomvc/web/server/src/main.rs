use anyhow::{anyhow, Result};
use failure::Fail;
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    AccessTokenHash, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
};
use serde::Deserialize;
use structopt::StructOpt;
use tide::{
    http::{
        cookies::{Cookie, SameSite},
        mime,
    },
    utils::After,
    Body, Error, Redirect, Request, Response, StatusCode,
};
use time::OffsetDateTime;
use url::Url;

mod async_client;

const HOME: &str = "../client/index.html";
const PKG: &str = "../client/pkg";
const PUBLIC: &str = "../client/public";

#[derive(Debug, Clone, StructOpt)]
struct Config {
    #[structopt(long, env = "API_URL")]
    api_url: Url,
    #[structopt(long, env = "WEB_URL")]
    web_url: Url,
    #[structopt(long, parse(from_str=client_id_from_str), env = "CLIENT_ID")]
    client_id: ClientId,
    #[structopt(long, parse(from_str=client_secret_from_str), env = "CLIENT_SECRET", hide_env_values = true)]
    client_secret: ClientSecret,
}

fn client_id_from_str(s: &str) -> ClientId {
    ClientId::new(s.to_string())
}

fn client_secret_from_str(s: &str) -> ClientSecret {
    ClientSecret::new(s.to_string())
}

#[derive(Clone)]
struct State {
    client: CoreClient,
    config: Config,
}

async fn home(req: Request<State>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);

    let body = Body::from_file(HOME).await?;
    res.set_body(body);

    let config = req.state().config.clone();
    let cookie = Cookie::build("api_url", config.api_url.to_string())
        .path("/")
        .same_site(SameSite::Strict)
        .finish();
    res.insert_cookie(cookie);

    res.set_content_type(mime::HTML);

    Ok(res)
}

async fn logout(_req: Request<State>) -> tide::Result {
    let mut res: Response = Redirect::new("/").into();
    let cookie = Cookie::build("token", "deleted")
        .path("/")
        .expires(OffsetDateTime::from_unix_timestamp(0))
        // .same_site(SameSite::Strict)
        // .http_only(true)
        // .secure(true)
        .finish();
    res.insert_cookie(cookie);

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

    let csrf_state = req
        .cookie("csrf_state")
        .ok_or_else(|| Error::from_str(StatusCode::Unauthorized, "no csrf state"))?;
    if query.state != csrf_state.value() {
        return Err(Error::from_str(StatusCode::Unauthorized, "bad csrf state"));
    }

    let pkce_code_verifier = req
        .cookie("pkce_code_verifier")
        .ok_or_else(|| Error::from_str(StatusCode::Unauthorized, "no PKCE code verifier"))?;
    let pkce_code_verifier = PkceCodeVerifier::new(pkce_code_verifier.value().to_string());

    let nonce = req
        .cookie("nonce")
        .ok_or_else(|| Error::from_str(StatusCode::Unauthorized, "no nonce"))?;
    let nonce = Nonce::new(nonce.value().to_string());

    let token_response = state
        .client
        .exchange_code(AuthorizationCode::new(query.code))
        .set_pkce_verifier(pkce_code_verifier)
        .request_async(async_client::async_http_client)
        .await
        .map_err(|e| {
            Error::from_str(
                StatusCode::Unauthorized,
                format!("cannot get token {:?}", e),
            )
        })?;

    let id_token_verifier = state.client.id_token_verifier();

    let id_token = token_response
        .extra_fields()
        .id_token()
        .ok_or_else(|| Error::from_str(StatusCode::Unauthorized, "no token"))?;

    let claims = id_token
        .claims(&id_token_verifier, &nonce)
        .map_err(|e| Error::new(StatusCode::Unauthorized, e.compat()))?;

    let expected_access_token_hash = claims
        .access_token_hash()
        .ok_or_else(|| Error::from_str(StatusCode::Unauthorized, "no token hash"))?;

    let alg = &id_token
        .signing_alg()
        .map_err(|e| Error::new(StatusCode::Unauthorized, e.compat()))?;

    let actual_access_token_hash = AccessTokenHash::from_token(token_response.access_token(), alg)
        .map_err(|e| Error::new(StatusCode::Unauthorized, e.compat()))?;

    if actual_access_token_hash != *expected_access_token_hash {
        return Ok(Response::new(StatusCode::Unauthorized));
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

    Ok(res)
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
    let mut redirect_url = config.web_url.clone();
    redirect_url.set_path("/callback");
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        config.client_id.clone(),
        Some(config.client_secret.clone()),
    )
    .set_redirect_uri(RedirectUrl::from_url(redirect_url));

    let mut app = tide::with_state(State { client, config });

    app.with(After(|res: Response| async move {
        match res.status() {
            StatusCode::Unauthorized => {
                if let Some(err) = res.error() {
                    let msg = err.to_string();
                    tide::log::error!("Unauthorized: {:?}", msg);
                }
                Ok(Response::new(StatusCode::Unauthorized))
            }
            _ => Ok(res),
        }
    }));

    app.at("/").get(home);
    app.at("/active").get(home);
    app.at("/completed").get(home);
    app.at("/healthz").get(|_| async { Ok(Response::new(204)) });
    app.at("/login").get(login);
    app.at("/logout").get(logout);
    app.at("/callback").get(callback);
    app.at("/pkg").serve_dir(PKG)?;
    app.at("/public").serve_dir(PUBLIC)?;

    app.listen("0.0.0.0:8080").await?;

    Ok(())
}
