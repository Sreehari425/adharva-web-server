
#[macro_use] extern crate rocket;
use std::os::linux::raw::stat;

use rocket::serde::{Serialize, json::Json};
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
    name: &'static str,
    status: EventStatus,
}

impl EventDetail{
    fn new(name:& 'static str , status: EventStatus) -> EventDetail {
        EventDetail { name , status}
    }
}




#[get("/api/v3/get/event-detail")]
fn event_detail() -> Json<EventDetail> {
    let event = EventDetail {
        name: "Treasure Hunt",
        status: EventStatus::Round1,
    };
    Json(event)
}




#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![event_detail])
}
