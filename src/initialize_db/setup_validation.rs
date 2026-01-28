use mongodb::{Collection, Database, IndexModel, bson::doc, options::{IndexOptions}};

use crate::user::add_user::User;

// async fn collection_exists(db: &Database, name: String)->bool{
//     let names = db.list_collection_names().await.unwrap();
//     if names.contains(&name){
//         true
//     }
//     else{
//         false
//     }
// }

pub async fn get_user_coll(db: Database)-> mongodb::error::Result<Collection<User>>{

        let keys = doc! {
            "email": 1
        };


        let options: IndexOptions = IndexOptions::builder()
        .unique(true)
        .build();

        let model: IndexModel= IndexModel::builder()
        .options(options)
        .keys(keys)
        .build();
        
        let coll: Collection<User> = db.collection::<User>("users");
        coll.create_index(model).await?;

        Ok(coll)

}