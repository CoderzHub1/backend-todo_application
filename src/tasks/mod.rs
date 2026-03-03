use crate::initialize_db::connect_db::connect;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: usize,
    pub name: String,
    pub status: bool,
    pub priority: usize,
}

pub mod add_task;
pub mod api;
pub mod get_tasks;
pub mod last_task;
pub mod remove_task;
pub mod update_task;

pub async fn get_user_tasks_coll(email: &str) -> Collection<Task> {
    let db = connect("tasks_todo").await;
    return db.collection::<Task>(email);
}
