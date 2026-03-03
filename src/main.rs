use axum::{Router, routing};
use my_api::{AppState, initialize_db::{connect_db::connect, setup_validation::get_users_coll}, tasks::{self, api::{self, TaskRemovalAPI, get_tasks_api}}, user::{add_user::create_user, auth_user::auth_user, get_user::get_user}};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main(){
    let users_db = connect("todo_userdata").await;
    let tasks_db = connect("todo_tasks").await;
    let users_coll = get_users_coll(users_db).await.unwrap();
    let state = AppState{
        users_coll,
        tasks_db
    };

    let app = Router::new()
    .route("/get-user", routing::get(get_user))
    .route("/create-user", routing::post(create_user))
    .route("/auth", routing::post(auth_user))
    .route("/add-task", routing::post(tasks::api::add_task_api))
    .route("/update-task", routing::post(api::udpate_task_api))
    .route("/get-tasks", routing::get(get_tasks_api))
    .route("/delete-task", routing::post(TaskRemovalAPI))
    .with_state(state)
    .layer(CorsLayer::permissive());
    
    println!("Server is now running on http://localhost:5050");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5050").await.expect("Can't listen to localhost:3000");
    
    axum::serve(listener, app).await.unwrap();

}
