use axum::{Json, extract::State};

use crate::{AppState, tasks::add_task::{TaskToAdd, add_task}};

#[axum::debug_handler]
pub async fn add_task_api(State(state):State<AppState>, Json(payload):Json<TaskToAdd>)->Json<Option<bool>>{
    let res: Option<bool> = add_task(state, payload).await.ok();
    Json(res)
}