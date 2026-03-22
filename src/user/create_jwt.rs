use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserJWT {
    email: String,
    expiration: usize
}


pub async fn create_jwt(email: String, secret: &String)-> Result<String, Box<dyn std::error::Error>> {
    let expiration = Utc::now().checked_add_signed(Duration::hours(24)).ok_or("Failed to fetch current time")?.timestamp() as usize;

    let claims = UserJWT{
        email,
        expiration
    };
    
    let jwt = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref())
    );

    return jwt.map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
}
