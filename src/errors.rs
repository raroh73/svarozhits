use askama::Template;
use axum::{
    body::{boxed, Full},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::error;

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundTemplate {}

#[derive(Template)]
#[template(path = "internal_server_error.html")]
struct InternalServerErrorTemplate {}

pub enum AppError {
    InternalServerError(anyhow::Error),
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::InternalServerError(err) => {
                error!("{:#}", err);

                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
                    .body(boxed(Full::from(
                        InternalServerErrorTemplate {}.render().unwrap(),
                    )))
                    .unwrap()
            }
            AppError::NotFound => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
                .body(boxed(Full::from(NotFoundTemplate {}.render().unwrap())))
                .unwrap(),
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AppError::InternalServerError(err.into())
    }
}
