use anyhow::anyhow;
use http_types::headers::{HeaderName, HeaderValue};
use openidconnect::{HttpRequest, HttpResponse};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Surf error {0:?}")]
    SurfError(surf::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

///
/// Asynchronous HTTP client.
///
pub async fn async_http_client(request: HttpRequest) -> Result<HttpResponse, RequestError> {
    let method =
        http_types::Method::from_str(request.method.as_str()).map_err(RequestError::SurfError)?;

    let body = request.body.clone();

    let mut req = surf::Request::new(method, request.url).set_body(&body[..]);

    for (name, value) in request.headers {
        let value = value.as_bytes().to_owned();
        let value = unsafe { HeaderValue::from_bytes_unchecked(value) };
        if let Some(name) = name {
            let name = name.as_str().as_bytes().to_owned();
            let name = unsafe { HeaderName::from_bytes_unchecked(name) };
            req.insert_header(name, value);
        }
    }

    tide::log::info!("request {:?}", req);
    let mut res = req.await.map_err(RequestError::SurfError)?;
    tide::log::info!("response {:?}", res);

    let status_code = openidconnect::http::StatusCode::from_u16(res.status().into())
        .map_err(|e| RequestError::Other(anyhow!("cannot convert status code: {:?}", e)))?;

    let mut headers = openidconnect::http::HeaderMap::new();
    for (name, values) in res.iter() {
        let name = format!("{}", name).into_bytes();
        if let Ok(name) = openidconnect::http::header::HeaderName::from_bytes(&name) {
            for value in values {
                let value = format!("{}", value).into_bytes();
                if let Ok(value) = openidconnect::http::header::HeaderValue::from_bytes(&value) {
                    headers.append(&name, value);
                }
            }
        }
    }

    let body: Vec<u8> = res
        .body_bytes()
        .await
        .map_err(|e| anyhow!("cannot read body bytes: {:?}", e))?;
    tide::log::info!("body: {:?}", std::str::from_utf8(&body));
    Ok(HttpResponse {
        status_code,
        headers,
        body,
    })
}
