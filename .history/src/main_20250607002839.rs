
#[macro_use] extern crate rocket;
use std::os::linux::raw::stat;

use rocket::{response::status, serde::{json::Json, Serialize}};
use chrono::{DateTime, Utc};
use std::fs;
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

fn load_events_from_file(path: &str) -> Vec<EventDetail> {
    let data = fs::read_to_string(path).expect("Failed to read events.json");
    serde_json::from_str(&data).expect("Failed to parse JSON")  
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
    let event = EventDetail {
        name: String::from("Treasure Hunt"),
        status: EventStatus::Round1,
    };
    Json(event)
}




#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![event_detail])
}
