use axum::{
    body::{boxed, Full},
    http::{header, StatusCode, Uri},
    response::Response,
};

use crate::models::Assets;

pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    match Assets::get(path) {
        Some(content) => {
            let body = boxed(Full::from(content.data));
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(body)
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(boxed(Full::from("404")))
            .unwrap(),
    }
}
