use askama::Template;
use axum::{
    body::{boxed, Empty, Full},
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::Response,
    Form,
};
use sqlx::SqlitePool;

use crate::models::{Assets, IndexTemplate, NotFoundTemplate, Task};

pub async fn show_index(State(db_pool): State<SqlitePool>) -> Response {
    let tasks = sqlx::query_as!(
        Task,
        "SELECT * FROM tasks WHERE task_status = 0 ORDER BY task_id DESC"
    )
    .fetch_all(&db_pool)
    .await
    .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
        .body(boxed(Full::from(IndexTemplate { tasks }.render().unwrap())))
        .unwrap()
}

pub async fn add_task(State(db_pool): State<SqlitePool>, Form(task): Form<Task>) -> Response {
    sqlx::query!(
        "INSERT INTO tasks (task_value) VALUES ($1)",
        task.task_value
    )
    .execute(&db_pool)
    .await
    .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("hx-refresh", "true")
        .body(boxed(Empty::new()))
        .unwrap()
}

pub async fn mark_task_as_done(
    State(db_pool): State<SqlitePool>,
    Path(task_id): Path<i64>,
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

pub async fn delete_task(State(db_pool): State<SqlitePool>, Path(task_id): Path<i64>) -> Response {
    sqlx::query!("DELETE FROM tasks WHERE task_id = $1", task_id)
        .execute(&db_pool)
        .await
        .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .body(boxed(Empty::new()))
        .unwrap()
}

pub async fn fallback(uri: Uri, headers: HeaderMap) -> Response {
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
            .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
            .body(boxed(Full::from(NotFoundTemplate {}.render().unwrap())))
            .unwrap(),
    }
}
