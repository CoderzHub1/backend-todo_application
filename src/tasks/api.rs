use std::cmp::{max};

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use crate::{AppState, tasks::{Task, add_task::{TaskToAdd, add_task}, get_tasks::get_tasks, update_task::update_task}};


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



// ---- Get tasks API

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTasks{
    email: String,
    counter: u16
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ResultGetTasks{
    pub res: bool,
    pub tasks: Option<Vec<Task>>,    
}

pub async fn get_tasks_api(State(state): State<AppState>, Json(payload): Json<GetTasks>)->Json<ResultGetTasks>{
    let res = get_tasks(payload.email, state, max(5, payload.counter)).await;
    match res{
        Ok(tasks)=>{
            return Json(ResultGetTasks { res: true, tasks:Some(tasks) });
        }
        Err(x)=>{
            eprintln!("{}",x);
            return Json(ResultGetTasks { res: false, tasks: None });
        }
    }

}
