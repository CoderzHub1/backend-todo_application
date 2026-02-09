use mongodb::{Collection, bson::{Document, doc}};
use crate::tasks::Task;

pub async fn update_task(id:u32, coll: Collection<Task>)->Result<bool, Box<dyn std::error::Error>>{
    let filter: Document = doc! {
        "id": id
    };
    
    let update: Document = doc! {
        "$set": {"status":true}
    };
    
    let res: Option<Task> = coll.find_one_and_update(filter, update).await?;

    match res{
        Some(x)=>{
            println!("{:#?}", x);
            Ok(true)
        }
        None=>{
            Ok(false)
        }
    }
}
