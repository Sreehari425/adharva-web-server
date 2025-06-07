#[macro_use] extern crate rocket;

use rocket::{serde::{json::Json, Serialize, Deserialize}};
use chrono::{DateTime, Utc};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct EventDetail {
    name: String,
    status: EventStatus,
}

fn load_events_from_file(path: &str) -> Vec<EventDetail> {
    let data = fs::read_to_string(path).expect("Failed to read events.json");
    serde_json::from_str(&data).expect("Failed to parse JSON")  
}

#[get("/api/v3/get/event-detail")]
fn event_detail() -> Json<Vec<EventDetail>> {
    let events = load_events_from_file("./events.json"); // make sure file exists
    Json(events)
}
#[post("/api/v3/update/<event_name>/<status>")]
fn update_event(event_name: &str, status: &str, state: &rocket::State<SharedEvents>) -> Json<Vec<EventDetail>> {
    use std::str::FromStr;

    let mut events = state.lock().unwrap();

    // Parse the string into an enum variant
    let parsed_status = match EventStatus::from_str(status) {
        Ok(s) => s,
        Err(_) => return Json(events.clone()), // optionally return error
    };

    for event in events.iter_mut() {
        if event.name == event_name {
            event.status = parsed_status.clone();
        }
    }

    // Save updated events to file
    fs::write("events.json", serde_json::to_string_pretty(&*events).unwrap()).expect("Unable to write file");

    Json(events.clone())
}
#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![event_detail])
}
