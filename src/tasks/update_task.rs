use mongodb::{Collection, bson::doc};
use crate::tasks::Task;

pub async fn update_task(id:u32, coll: Collection<Task>)->Result<bool, Box<dyn std::error::Error>>{
    let filter = doc! {
        "id": id
    };
    
    let update = doc! {
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
