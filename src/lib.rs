use mongodb::{Collection, Database};

use crate::user::add_user::User;

pub mod tasks;
pub mod initialize_db;
pub mod user;

#[derive(Clone)]
pub struct AppState{
    pub users_coll: Collection<User>,
    pub tasks_db: Database,
    pub jwt_secret: String,
}
