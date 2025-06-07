
#[macro_use] extern crate rocket;
use std::os::linux::raw::stat;

use rocket::{response::status, serde::{json::Json, Serialize}};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
enum EventStatus {
    Started,
    Ended,
    Round1,
    Round2,
    Round3,
    Round4,
    Ongoing,
    Delayed,
    Soon,
}

#[derive(Debug, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
struct EventDetail {
    name: String,
    status: EventStatus,
}

impl EventDetail{
    fn new(name:String , status: EventStatus) -> EventDetail {
        EventDetail { name , status}
    }
    fn set_status(mut self,status: EventStatus){
        self.status = status
    }
}




#[get("/api/v3/get/event-detail")]
fn event_detail() -> Json<EventDetail> {
    ... 
}




#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![event_detail])
}
