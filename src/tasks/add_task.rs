use crate::{AppState, tasks::{Task, last_task::get_last_task_id}, user::check_user_access::check_user_access};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::{error::Error};

#[derive(Serialize, Deserialize)]
pub struct TaskToAdd {
    pub name: String,
    pub priority: usize,
    pub email: String,
    pub auth_pass: String,
}

pub async fn add_task(state: AppState, payload: TaskToAdd) -> Result<bool, Box<dyn Error>> {
    let user: Option<crate::user::add_user::User> = state
        .users_coll
        .find_one(doc! {"email": &payload.email})
        .await?;
    
    match user {
        None => Ok(false),

        Some(user) => {

            let user_found: crate::user::add_user::User = user;
            let coll: mongodb::Collection<Task> = state.tasks_db.collection(&user_found.email);
            let auth = check_user_access(&user_found.email, &payload.auth_pass, state.users_coll).await?;
            if auth {
                let id = get_last_task_id(&coll).await?+1;
                let new_task = Task{
                    id,
                    name: payload.name,
                    status: false,
                    priority: payload.priority
                };
                println!("{:#?}", new_task);
               let _status = coll
                    .insert_one(new_task)
                    .await?;

                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}
