use axum::{Json, extract::State};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{AppState};

#[derive(Serialize, Deserialize)]
pub struct UserAuth {
    email: String,
    pass: String,
}


#[derive(Serialize, Deserialize)]
pub struct Res {
    auth: bool,
    error: bool,
}

pub async fn auth_user(State(state): State<AppState>, Json(payload): Json<UserAuth>) -> Json<Res> {
    let user = state
        .users_coll
        .find_one(doc! {"email": payload.email})
        .await;
    match user {
        Err(x) => {
            eprintln!("\n Error at auth_user: {}", x);
            return Json(Res {
                auth: false,
                error: true,
            });
        }

        Ok(None) => {
            return Json(Res {
                auth: false,
                error: false,
            });
        }

        Ok(Some(user)) => {
            let auth = bcrypt::verify(payload.pass, &user.password);
            match auth {
                Ok(val) => Json(Res {
                    auth: val,
                    error: false,
                }),
                Err(x) => {
                    eprintln!("\n Error at auth_user: {}", x);
                    Json(Res {
                    auth: false,
                    error: true,
                })
            }
            }
        }
    }
}
