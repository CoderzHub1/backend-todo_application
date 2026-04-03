use mongodb::{Collection, bson::{Document, doc}};
use crate::tasks::Task;

pub async fn toggle_task_state(id:u32, coll: Collection<Task>)->Result<bool, Box<dyn std::error::Error>>{
    let filter: Document = doc! {
        "id": id
    };

    let task = match coll.find_one(filter.clone()).await? {
        Some(val) => val,

        None => {
            return Err("task not found".into());
        }
    };
    
    let update: Document = doc! {
        "$set": {"status": !task.status}
    };
    
    let res: Option<Task> = coll.find_one_and_update(filter, update).await?;

    match res{
        Some(_x)=>{
            return Ok(true);
        }
        None=>{
            return Ok(false);
        }
    }
}
