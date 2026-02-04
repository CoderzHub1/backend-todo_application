use mongodb::{Collection, bson::doc};
use mongodb::options::FindOneOptions;

use crate::tasks::{Task};

pub async fn get_last_task_id(coll: &Collection<Task>) -> mongodb::error::Result<usize> {

    let filter = doc! {};

    let options: FindOneOptions = FindOneOptions::builder().sort(doc! { "id": -1 }).build();

    let res = coll.find_one(filter).with_options(Some(options)).await?;
    
    let count = coll.count_documents(doc! {}).await?;
    println!("DOC COUNT = {}", count);

    match res {
        Some(x) => Ok(x.id),
        None => Ok(0),
    }
}
