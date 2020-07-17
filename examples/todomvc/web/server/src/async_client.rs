use oauth2::{HttpRequest, HttpResponse};
use surf::http::headers::{HeaderName, HeaderValue};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("unknown method {0:?}")]
    SurfError(surf::Error),
}

///
/// Asynchronous HTTP client.
///
pub async fn async_http_client(request: HttpRequest) -> Result<HttpResponse, RequestError> {
    let method = surf::http::Method::from(request.method);

    let body = request.body.clone();

    let mut req = surf::Request::new(method, request.url).body_bytes(&body[..]);

    for (name, value) in request.headers {
        let value = value.as_bytes().to_owned();
        let value = unsafe { HeaderValue::from_bytes_unchecked(value) };
        let name = name.unwrap().as_str().as_bytes().to_owned();
        let name = unsafe { HeaderName::from_bytes_unchecked(name) };
        req.insert_header(name, value);
    }

    tide::log::info!("request {:?}", req);
    let mut res = req.await.map_err(RequestError::SurfError)?;
    tide::log::info!("response {:?}", res);

    let status_code = res.status().into();

    let mut headers = oauth2::http::HeaderMap::new();
    for (name, values) in res.iter() {
        let name = format!("{}", name).into_bytes();
        if let Ok(name) = oauth2::http::header::HeaderName::from_bytes(&name) {
            for value in values {
                let value = format!("{}", value).into_bytes();
                if let Ok(value) = oauth2::http::header::HeaderValue::from_bytes(&value) {
                    headers.append(&name, value);
                }
            }
        }
    }

    let body: Vec<u8> = res.body_bytes().await.map_err(RequestError::SurfError)?;
    tide::log::info!("body: {:?}", std::str::from_utf8(&body));
    Ok(HttpResponse {
        status_code,
        headers,
        body,
    })
}
