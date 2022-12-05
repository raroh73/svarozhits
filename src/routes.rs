use anyhow::Context;
use askama::Template;
use axum::{
    body::{boxed, Empty, Full},
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode, Uri},
    response::Response,
    Form,
};
use sqlx::SqlitePool;

use crate::{
    errors::AppError,
    models::{Assets, IndexTemplate, Task},
};

pub async fn show_index(State(db_pool): State<SqlitePool>) -> Result<Response, AppError> {
    let tasks = sqlx::query_as!(
        Task,
        "SELECT * FROM tasks WHERE task_status = 0 ORDER BY task_id DESC"
    )
    .fetch_all(&db_pool)
    .await
    .context("show_index")?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime::TEXT_HTML.as_ref())
        .body(boxed(Full::from(
            IndexTemplate { tasks }.render().context("show_index")?,
        )))
        .context("show_index")?)
}

pub async fn add_task(
    State(db_pool): State<SqlitePool>,
    Form(task): Form<Task>,
) -> Result<Response, AppError> {
    sqlx::query!(
        "INSERT INTO tasks (task_value) VALUES ($1)",
        task.task_value
    )
    .execute(&db_pool)
    .await
    .context("add_task")?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("hx-refresh", "true")
        .body(boxed(Empty::new()))
        .context("add_task")?)
}

pub async fn mark_task_as_done(
    State(db_pool): State<SqlitePool>,
    Path(task_id): Path<i64>,
) -> Result<Response, AppError> {
    sqlx::query!(
        "UPDATE tasks SET task_status = 1 WHERE task_id = $1",
        task_id
    )
    .execute(&db_pool)
    .await
    .context("mark_task_as_done")?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(boxed(Empty::new()))
        .context("mark_task_as_done")?)
}

pub async fn delete_task(
    State(db_pool): State<SqlitePool>,
    Path(task_id): Path<i64>,
) -> Result<Response, AppError> {
    sqlx::query!("DELETE FROM tasks WHERE task_id = $1", task_id)
        .execute(&db_pool)
        .await
        .context("delete_task")?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(boxed(Empty::new()))
        .context("delete_task")?)
}

pub async fn fallback(uri: Uri, headers: HeaderMap) -> Result<Response, AppError> {
    let path = uri.path().trim_start_matches('/');

    let file = Assets::get(path).ok_or(AppError::NotFound)?;
    let hash = hex::encode(file.metadata.sha256_hash());

    if headers
        .get(header::IF_NONE_MATCH)
        .map(|etag| etag.to_str().unwrap_or("000000").eq(&hash))
        .unwrap_or(false)
    {
        return Ok(Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .body(boxed(Empty::new()))
            .context("fallback")?);
    }

    let body = boxed(Full::from(file.data));
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .header(header::ETAG, hash)
        .body(body)
        .context("fallback")?)
}
