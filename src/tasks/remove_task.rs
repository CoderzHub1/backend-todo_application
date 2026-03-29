use mongodb::bson::doc;

use crate::{AppState, tasks::Task, user::{jwt::verify_jwt}};

pub async fn remove_task(auth_jwt: &String, id: u32, state:AppState)-> Result<bool, Box<dyn std::error::Error>> {
    
    let user_data = verify_jwt(&auth_jwt, &state.jwt_secret).await?;

    let user = state.users_coll.find_one(doc! {"email": &user_data.claims.email}).await?;

    match user {
        Some(_x)=>{
            let coll:mongodb::Collection<Task> = state.tasks_db.collection(&user_data.claims.email);
            let _res = coll.find_one_and_delete(doc! {"id": &id}).await?;
            Ok(true)
        }
        None=>{
            Ok(false)
        }
    }
}
