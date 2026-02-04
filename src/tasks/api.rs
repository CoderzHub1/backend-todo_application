use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use crate::{AppState, tasks::{add_task::{TaskToAdd, add_task}, update_task::update_task}};


// ----- Add Task API
#[axum::debug_handler]
pub async fn add_task_api(State(state):State<AppState>, Json(payload):Json<TaskToAdd>)->Json<Option<bool>>{
    let res: Option<bool> = add_task(state, payload).await.ok();
    Json(res)
}
// ----- Task Update API
#[derive(Serialize, Deserialize, Debug)]
pub struct TaskUpdate{
    id: u32,
    email: String
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Res{
    pub success: bool
}

pub async fn udpate_task_api(State(state): State<AppState>, Json(payload):Json<TaskUpdate>)->Json<Res>{
    let coll: mongodb::Collection<super::Task> = state.tasks_db.collection(&payload.email);
    let res: Result<bool, Box<dyn std::error::Error>> = update_task(payload.id, coll).await;

    match res{
        Ok(_x)=>{
            return Json(Res{
                success: true
            });
        }
        Err(x)=>{
            eprintln!("{}", x);
            return Json(Res { success: false });
        }
    }
}
