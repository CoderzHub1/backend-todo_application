use std::cmp::max;

use crate::{
    AppState,
    tasks::{
        Task,
        add_task::{TaskToAdd, add_task},
        get_tasks::get_tasks,
        remove_task::remove_task,
        update_task::update_task,
    }, user::jwt::verify_jwt,
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

// ----- Add Task API
#[axum::debug_handler]
pub async fn add_task_api(
    State(state): State<AppState>,
    Json(payload): Json<TaskToAdd>,
) -> Json<bool> {
    let res = match add_task(state, payload).await {
        Ok(val)=>val,
        Err(x)=>{
            eprintln!("Error while adding task: {}", {x} ); 
            false
        }
    };
    Json(res)
}
// ----- Task Update API
#[derive(Serialize, Deserialize, Debug)]
pub struct TaskUpdate {
    id: u32,
    auth_jwt: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Res {
    pub success: bool,
}

pub async fn udpate_task_api(
    State(state): State<AppState>,
    Json(payload): Json<TaskUpdate>,
) -> impl IntoResponse {
    
    let user = match verify_jwt(&payload.auth_jwt, &state.jwt_secret).await {
        Ok(val) => val,
        Err(_x) => {
            return (StatusCode::UNAUTHORIZED, Json(Res{success:false}));
        }
    };


    let coll: mongodb::Collection<super::Task> = state.tasks_db.collection(&user.claims.email);
    let res = update_task(payload.id, coll).await;

    match res {
        Ok(val) => {
            return (StatusCode::OK, Json(Res{success: val}));
        }
        Err(x) => {
            eprintln!("{}", x);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(Res{success: false}));
        }
    }
}

// ---- Get tasks API

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTasks {
    auth_jwt: String,
    counter: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultGetTasks {
    pub res: bool,
    pub tasks: Option<Vec<Task>>,
}

pub async fn get_tasks_api(
    State(state): State<AppState>,
    Json(payload): Json<GetTasks>,
) -> Json<ResultGetTasks> {
    let res = get_tasks(payload.auth_jwt, state, max(5, payload.counter)).await;

    match res {
        Ok(tasks) => {
            return Json(ResultGetTasks {
                res: true,
                tasks: Some(tasks),
            });
        }
        Err(x) => {
            eprintln!("{}", x);
            return Json(ResultGetTasks {
                res: false,
                tasks: None,
            });
        }
    }
}

// ------- Task removal api

#[derive(Serialize, Deserialize)]
pub struct TaskRemovalPayload {
    pub auth_jwt: String,
    pub task_id: u32,
}

pub async fn task_removal_api(State(state): State<AppState>, Json(payload): Json<TaskRemovalPayload>) -> impl IntoResponse {
    let res = remove_task(&payload.auth_jwt, payload.task_id, state).await;

    match res {
        Ok(x) => (StatusCode::OK, Json(Res { success: x })),
        Err(x) => {
            eprintln!("Error while removing a task, user_jwt: {}, Error: {}", payload.auth_jwt, x);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(Res { success: false }),
            );

        }
    }
}
