use axum::{Json, extract::State};
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};

use crate::AppState;


#[derive(Serialize, Deserialize)]
pub struct UserUnhashed{
    username: String,
    email: String,
    password: String
}


#[derive(Serialize, Deserialize, Debug)]
pub struct User{
    pub username: String,
    pub email: String,
    pub password: String
}

#[derive(Serialize, Deserialize)]
pub struct Status{
    pub status: String
}



pub async fn create_user(State(state): State<AppState>, Json(payload): Json<UserUnhashed>) -> Json<Status>{
    let coll: mongodb::Collection<User> = state.users_coll;

    let mut user = User{
            username: payload.username,
            email: payload.email,
            password: String::from("")
    };

    {
        let hashed_pass = hash(payload.password, DEFAULT_COST);

        match hashed_pass {
            Ok(pass)=>{
                user.password = pass;
            
        }
        Err(_x)=>{
            eprintln!("Bcrypt Error while adding user (add_user.rs): {}", _x);
            return Json(Status {status: format!("Error raised while creating the user (pass-hashing)")});
        }   
        }
    }
    let success = coll.insert_one(&user).await;
    match success {
        Ok(_x)=>{
            println!("User added successfully\n{:#?}", user);
            return Json(Status{status: String::from("Success")});
        }
        Err(_x)=>{
            eprintln!("MongoDB error while inserting user (add_user.rs): {}", _x);
            return Json(Status {status: format!("Error occured while adding user")});
        }
    }
}