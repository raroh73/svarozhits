use askama_axum::IntoResponse;
use axum::{
    body::{boxed, Empty, Full},
    extract::Path,
    http::{header, HeaderMap, StatusCode, Uri},
    response::Response,
    Extension, Form,
};
use sqlx::SqlitePool;

use crate::models::{Assets, IndexTemplate, Task};

pub async fn show_index(Extension(db_pool): Extension<SqlitePool>) -> IndexTemplate {
    let tasks = sqlx::query_as!(Task, "SELECT * FROM tasks WHERE task_status = 0")
        .fetch_all(&db_pool)
        .await
        .unwrap();
    IndexTemplate { tasks }
}

pub async fn add_task(
    Form(task): Form<Task>,
    Extension(db_pool): Extension<SqlitePool>,
) -> Response {
    sqlx::query!(
        "INSERT INTO tasks (task_value) VALUES ($1)",
        task.task_value
    )
    .execute(&db_pool)
    .await
    .unwrap();

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/")
        .body(boxed(Empty::new()))
        .unwrap()
}

pub async fn mark_task_as_done(
    Path(task_id): Path<i64>,
    Extension(db_pool): Extension<SqlitePool>,
) -> Response {
    sqlx::query!(
        "UPDATE tasks SET task_status = 1 WHERE task_id = $1",
        task_id
    )
    .execute(&db_pool)
    .await
    .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .body(boxed(Empty::new()))
        .unwrap()
}

pub async fn delete_task(
    Path(task_id): Path<i64>,
    Extension(db_pool): Extension<SqlitePool>,
) -> Response {
    sqlx::query!("DELETE FROM tasks WHERE task_id = $1", task_id)
        .execute(&db_pool)
        .await
        .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .body(boxed(Empty::new()))
        .unwrap()
}

pub async fn static_handler(uri: Uri, headers: HeaderMap) -> Response {
    let path = uri.path().trim_start_matches('/');

    match Assets::get(path) {
        Some(content) => {
            let hash = hex::encode(content.metadata.sha256_hash());

            if headers
                .get(header::IF_NONE_MATCH)
                .map(|etag| etag.to_str().unwrap_or("000000").eq(&hash))
                .unwrap_or(false)
            {
                return Response::builder()
                    .status(StatusCode::NOT_MODIFIED)
                    .body(boxed(Empty::new()))
                    .unwrap();
            }

            let body = boxed(Full::from(content.data));
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::ETAG, hash)
                .body(body)
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(boxed(Full::from("404")))
            .unwrap(),
    }
}
