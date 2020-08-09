use anyhow::{anyhow, Result};
use base64::decode;
use serde::Deserialize;
use std::str;

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub name: String,
}

pub fn decode_jwt(token: &str) -> Result<Claims> {
    let parts: Vec<&str> = token.split('.').collect();
    let claims = parts.get(1).ok_or_else(|| anyhow!("malformed token"))?;
    let claims = decode(claims)?;
    let claims = str::from_utf8(&claims[..])?;
    let claims = serde_json::from_str(claims)?;
    Ok(claims)
}
