use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{AppState, user::jwt::verify_jwt};

#[derive(Serialize, Deserialize)]
pub struct JwtPayload {
	pub jwt: String,
}

#[derive(Serialize, Deserialize)]
pub struct JwtAuthRes {
	pub email: Option<String>,
	pub tampered: bool,
}

pub async fn auth_user_jwt(
	State(state): State<AppState>,
	Json(payload): Json<JwtPayload>,
) -> Json<JwtAuthRes> {
	match verify_jwt(&payload.jwt, &state.jwt_secret).await {
		Ok(token_data) => Json(JwtAuthRes {
			email: Some(token_data.claims.email),
			tampered: false,
		}),
		Err(_x) => Json(JwtAuthRes {
			email: None,
			tampered: true,
		}),
	}
}
