use askama_axum::IntoResponse;
use axum::{
    body::{boxed, Full},
    extract::Path,
    http::{header, StatusCode, Uri},
    response::{Redirect, Response},
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
) -> Redirect {
    sqlx::query!(
        "INSERT INTO tasks (task_value) VALUES ($1)",
        task.task_value
    )
    .execute(&db_pool)
    .await
    .unwrap();

    Redirect::to("/")
}

pub async fn mark_task_as_done(
    Path(task_id): Path<i64>,
    Extension(db_pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    sqlx::query!(
        "UPDATE tasks SET task_status = 1 WHERE task_id = $1",
        task_id
    )
    .execute(&db_pool)
    .await
    .unwrap();

    StatusCode::OK
}

pub async fn delete_task(
    Path(task_id): Path<i64>,
    Extension(db_pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    sqlx::query!("DELETE FROM tasks WHERE task_id = $1", task_id)
        .execute(&db_pool)
        .await
        .unwrap();

    StatusCode::OK
}

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
