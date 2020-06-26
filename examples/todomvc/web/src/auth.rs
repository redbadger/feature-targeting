use anyhow::{anyhow, Result};
use base64::decode;
use serde::{Deserialize, Serialize};
use std::str;
use url::Url;
use uuid::Uuid;

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
    #[serde(rename = "authuser")]
    pub auth_user: i32,
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
    let parts: Vec<&str> = token.split('.').collect();
    let claims = parts.get(1).ok_or_else(|| anyhow!("malformed token"))?;
    let claims = decode(claims)?;
    let claims = str::from_utf8(&claims[..])?;
    let claims = serde_json::from_str(claims)?;
    Ok(claims)
}
