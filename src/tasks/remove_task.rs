use mongodb::bson::doc;

use crate::{AppState, tasks::Task, user::check_user_access::check_user_access};

pub async fn remove_task(email: String, auth_pass: String, id: u32, state:AppState)-> Result<bool, Box<dyn std::error::Error>> {
    let user = state.users_coll.find_one(doc! {"email": &email}).await?;

    match user {
        Some(_x)=>{
            if check_user_access(&email, &auth_pass, state.users_coll).await?  != true {
                Ok(false)
            }
            else {
                let coll:mongodb::Collection<Task> = state.tasks_db.collection(&email);
                let res = coll.find_one_and_delete(doc! {"id": &id}).await?;
                println!("Deleted document: {:#?}", res);
                Ok(true)
            }
        }
        None=>{
            Ok(false)
        }
    }
}
