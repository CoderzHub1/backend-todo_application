use mongodb::Collection;

use crate::user::add_user::User;

pub mod initialize_db;
pub mod user;

#[derive(Clone)]
pub struct AppState{
    pub users_coll: Collection<User>
}