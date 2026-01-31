use mongodb::{Collection, bson::{doc}};
use std::error::Error;
use crate::user::add_user::User;

pub async fn check_user_access(email: &String, pass:&String, users_coll:Collection<User>)-> Result<bool, Box<dyn Error>>{
    let user = users_coll
        .find_one(doc! {"email": email})
        .await?;
    match user {

        None => {
            return Ok(false)
        }

        Some(user) => {
            return Ok(bcrypt::verify(&pass, &user.password)?);
        }
    }
}