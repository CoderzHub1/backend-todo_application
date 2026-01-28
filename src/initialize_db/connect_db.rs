use mongodb::{Client, Database};

pub async fn connect()-> Database{
    let uri = "mongodb://127.0.0.1:27017";
    let client:Client = Client::with_uri_str(uri).await.expect("Can't connect to mongodb with uri {uri}");

    let db:Database = client.database("my_api_learning");

    return db;
}
