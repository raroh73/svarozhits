use askama::Template;
use rust_embed::RustEmbed;
use serde::Deserialize;

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub tasks: Vec<Task>,
}

#[derive(Deserialize)]
pub struct Task {
    #[serde(default)]
    pub task_id: i64,
    pub task_value: String,
    #[serde(default)]
    pub task_status: i64,
}
