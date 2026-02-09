use mongodb::bson::doc;

use crate::{AppState, tasks::Task};

pub async fn get_tasks(
    email: String,
    state: AppState,
    counter: u16,
) -> Result<Vec<Task>, mongodb::error::Error> {
    let coll: mongodb::Collection<Task> = state.tasks_db.collection(&email);
    let mut res = coll.find(doc! {}).await?;
    let mut tasks: Vec<Task> = vec![];
    let mut iterations = 0;
    while res.advance().await? {
        let task = res.deserialize_current()?;
        tasks.push(task);
        iterations += 1;

        if counter >= iterations {
            break;
        }
    }
    return Ok(tasks);
}