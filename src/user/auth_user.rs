use axum::{Json, extract::State};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{AppState, user::{check_user_access::check_user_access, create_jwt::{self, create_jwt}}};

#[derive(Serialize, Deserialize)]
pub struct UserAuth {
    email: String,
    pass: String,
}


#[derive(Serialize, Deserialize)]
pub struct Res {
    auth: bool,
    error: bool,
    jwt: Option<String>,
}

pub async fn auth_user(State(state): State<AppState>, Json(payload): Json<UserAuth>)-> Json<Res>{
    let auth = match check_user_access(&payload.email, &payload.pass, state.users_coll).await {
        Ok(val)=> val,
        Err(x)=>{
            eprintln!("Error at auth_user {}", x);
            return Json(Res{
                auth: false,
                error: true,
                jwt: None
            })
        }
    };

    if auth == false {
        return Json(Res{
            auth: false,
            error: true,
            jwt: None
        });
    }

    let jwt = match create_jwt(payload.email, &state.jwt_secret).await {
        Ok(val) => val,

        Err(x)=> {
            eprintln!("JWT Auth error: {}", x);
            return Json(Res{
                auth: false,
                error: true,
                jwt: None
            })
        }
    };

    return Json(Res{
        auth: true,
        error: false,
        jwt: Some(jwt)
    });
}
