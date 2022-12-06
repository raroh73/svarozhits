use askama::Template;
use axum::{
    body::{boxed, Full},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::error;

pub enum AppError {
    NotFound,
    InternalServerError(anyhow::Error),
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AppError::InternalServerError(err.into())
    }
}

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundTemplate {}

#[derive(Template)]
#[template(path = "internal_server_error.html")]
struct InternalServerErrorTemplate {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
                .body(boxed(Full::from(
                    NotFoundTemplate {}
                        .render()
                        .unwrap_or_else(|_| "<h1>NOT FOUND</h1>".to_string()),
                )))
                .unwrap_or_else(|_| (StatusCode::NOT_FOUND, "NOT FOUND").into_response()),
            AppError::InternalServerError(err) => {
                error!("{:#}", err);

                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
                    .body(boxed(Full::from(
                        InternalServerErrorTemplate {}
                            .render()
                            .unwrap_or_else(|_| "<h1>INTERNAL SERVER ERROR</h1>".to_string()),
                    )))
                    .unwrap_or_else(|_| {
                        (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL SERVER ERROR").into_response()
                    })
            }
        }
    }
}
