use anyhow::{anyhow, Result};
// use jsonwebtoken::{decode, DecodingKey, Validation};
use base64::decode;
use serde::{Deserialize, Serialize};
use std::str;
use url::Url;
use uuid::Uuid;

// const JKWS_URL: &str = "https://accounts.google.com/.well-known/openid-configuration";

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub state: String,
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
    pub scope: String,
    pub id_token: String,
    pub authuser: i32,
    pub hd: String,
    pub prompt: String,
}

pub fn get_login_url() -> Result<Url> {
    let oauth2_endpoint = Url::parse_with_params(
        "https://accounts.google.com/o/oauth2/v2/auth",
        &[
            (
                "client_id",
                "370114826193-18ss0t0cqlqqredmifhuu27itlpajg9s.apps.googleusercontent.com",
            ),
            ("redirect_uri", "http://localhost:8080"),
            ("response_type", "token id_token"),
            ("nonce", Uuid::new_v4().to_string().as_ref()),
            ("scope", "https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/userinfo.profile openid"),
            ("include_granted_scopes", "true"),
            ("state", "pass-through value"),
        ],
    )?;
    Ok(oauth2_endpoint)
}

pub fn logout() -> Result<()> {
    Ok(())
}

pub fn get_claims(auth_response: &AuthResponse) -> Result<Claims> {
    let token = auth_response.id_token.clone();
    decode_jwt(token.as_str())
}

fn decode_jwt(token: &str) -> Result<Claims> {
    // let pubkey = "7edwjFTS82NuGM29NNFkFvd0nGSSaCGJkM7MqXQyha1iz7DFVa2pMOboAv7NoGd9mbmwMDrAAOeP88U1WPnmybkpFIszvxidakkyHTE_UfJgtLo456ck1u18UwQXwOJprCFmkpOd9dzEbx4L2YwxWNXQzTl8k-7yRFuiJrfJrsLrxa8r-eZJAzgxVzgRQp_AyTTqRgUi9sC4p6m5BuFi-2xr2_2a0Z9qgpQ6hxsSVyo2jmnVQ4rBmNdKCDIR4FBVP5NmVDlFNOpRauzwKGa2VPHcbOqKVlFHRd43NGgTMXZVfsSghy5UoLr4eKYMA3LeFszcWarhNxz_-wqcwx3h8w";
    // let key = DecodingKey::from_secret(pubkey.as_ref());
    // super::log!(key);
    // let token = decode::<Claims>(&token, &key, &Validation::default()).unwrap();
    // todo!()
    let parts: Vec<&str> = token.split('.').collect();
    let claims = parts.get(1).ok_or_else(|| anyhow!("malformed token"))?;
    let claims = decode(claims)?;
    let claims = str::from_utf8(&claims[..])?;
    let claims = serde_json::from_str(claims)?;
    Ok(claims)
}
