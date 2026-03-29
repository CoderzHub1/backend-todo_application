use crate::{AppState, tasks::{Task, last_task::get_last_task_id}, user::{jwt}};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::{error::Error};

#[derive(Serialize, Deserialize)]
pub struct TaskToAdd {
    pub name: String,
    pub priority: usize,
    pub auth_jwt: String,
}

pub async fn add_task(state: AppState, payload: TaskToAdd) -> Result<bool, Box<dyn Error>> {
    let user_data = jwt::verify_jwt(&payload.auth_jwt, &state.jwt_secret).await?;

    let user: Option<crate::user::add_user::User> = state
        .users_coll
        .find_one(doc! {"email": user_data.claims.email})
        .await?;
    
    match user {
        None => Ok(false),

        Some(user) => {

            let user_found = user;
            let coll: mongodb::Collection<Task> = state.tasks_db.collection(&user_found.email);

            let id = get_last_task_id(&coll).await?+1;
            let new_task = Task{
                id,
                name: payload.name,
                status: false,
                priority: payload.priority
            };

           let _status = coll
                .insert_one(new_task)
                .await?;

            Ok(true)
        }
    }
}
