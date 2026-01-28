use axum::{Router, routing};
use my_api::{AppState, initialize_db::{connect_db::connect, setup_validation::get_user_coll}, user::{add_user::create_user, auth_user::auth_user, get_user::get_user}};
use tower_http::cors::CorsLayer;


#[tokio::main]
async fn main(){
    let db = connect().await;
    let users_coll = get_user_coll(db).await.unwrap();
    let state = AppState{
        users_coll
    };

    let app = Router::new()
    .route("/get-user", routing::get(get_user))
    .route("/create-user", routing::post(create_user))
    .route("/auth", routing::post(auth_user))
    .with_state(state)
    .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.expect("Can't listen to localhost:3000");
    axum::serve(listener, app).await.unwrap();

}
