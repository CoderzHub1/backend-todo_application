use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserJWT {
    pub email: String,
    #[serde(rename = "exp")]
    pub expiration: usize,
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


pub async fn verify_jwt(token: &String, secret: &String)->Result<TokenData<UserJWT>, jsonwebtoken::errors::Error>{
    return decode::<UserJWT>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default()
    );
}
