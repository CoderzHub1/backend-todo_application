use std::collections::HashMap;

use axum::{Json, extract::{Query, State}};
use mongodb::{bson::doc};
use serde::{Deserialize, Serialize};

use crate::{AppState};

#[derive(Serialize, Deserialize)]
pub struct CensoredUser{
    pub username: String,
    pub email: String
}
#[axum::debug_handler]
pub async fn get_user( State(state): State<AppState>, Query(params): Query<HashMap<String, String>> ) -> Json<CensoredUser> {
    let check_email = params.get("email");
    match check_email{
        Some(email)=>{
            let status = state.users_coll.find_one(doc! {"email": email}).await;

            match status{
                Ok(Some(x))=>{
                    let user  = CensoredUser{
                        username: x.username,
                        email: x.email
                    };

                    return Json(user);
                }

                Ok(None)=>{
                    return Json(CensoredUser { username: "".to_string(), email: "".to_string() });
                }

                Err(x)=>{
                    eprintln!("\nError `get_user()`: {}", x);
                    return Json(CensoredUser { username: "".to_string(), email: "".to_string() });
                }
            }
            
        }
        None=>{
            Json(CensoredUser { username: "".to_string(), email: "".to_string() })
        }
    }
}