use crate::{AppState, tasks::Task, user::check_user_access::check_user_access};
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
        .find_one(doc! {"email": payload.email})
        .await?;

    match user {
        None => Ok(false),

        Some(user) => {

            let user_found: crate::user::add_user::User = user;
            let coll: mongodb::Collection<Task> = state.tasks_db.collection(&user_found.email);
            let auth = check_user_access(&user_found.email, &payload.auth_pass, state.users_coll).await?;
            if auth {
                let _status = coll
                    .insert_one(Task {
                        name: payload.name,
                        status: false,
                        priority: payload.priority,
                    })
                    .await?;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}
