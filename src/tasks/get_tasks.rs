use std::error::Error;
use mongodb::bson::doc;
use crate::{AppState, tasks::Task, user::jwt::verify_jwt};

pub async fn get_tasks(auth_jwt: String, state: AppState, counter: u16) -> Result<Vec<Task>,Box<dyn Error>> { 
    
    let user_data = verify_jwt(&auth_jwt, &state.jwt_secret).await?;

    let coll: mongodb::Collection<Task> = state.tasks_db.collection(&user_data.claims.email);
    let mut res = coll.find(doc! {}).await?;
    let mut tasks: Vec<Task> = vec![];
    let mut iterations:u16 = 0;
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
