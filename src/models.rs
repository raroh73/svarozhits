use rust_embed::RustEmbed;
use serde::Deserialize;

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

#[derive(Deserialize)]
pub struct Task {
    #[serde(default)]
    pub task_id: i64,
    pub task_value: String,
    #[serde(default)]
    pub task_status: i64,
}
