
#[macro_use] extern crate rocket;
use rocket::serde::{Serialize};
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
}

#[derive(Debug, Clone, Serialize)]
#[serde(crate = "rocket::serde")]
struct EventDetail {
    name: &'static str,
    status: EventStatus,
    time: DateTime<Utc>,
    description: String,
}

#[get("/api/v3/get/event-detail")]
fn event_detail() -> Json<EventDetail> {
    let event = EventDetail {
        name: "Treasure Hunt",
        status: EventStatus::Round1,
        time: Utc::now(),
        description: "Round 1 is live!".into(),
    };
    Json(event)
}
#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![event_detail])
}
