#[macro_use] extern crate rocket;

use rocket::{serde::{json::Json, Serialize, Deserialize}};
use chrono::{DateTime, Utc};
use std::fs;
use std::str::FromStr;
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


impl FromStr for EventStatus {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "started" => Ok(EventStatus::Started),
            "ended" => Ok(EventStatus::Ended),
            "round1" => Ok(EventStatus::Round1),
            "round2" => Ok(EventStatus::Round2),
            "round3" => Ok(EventStatus::Round3),
            "round4" => Ok(EventStatus::Round4),
            "ongoing" => Ok(EventStatus::Ongoing),
            "delayed" => Ok(EventStatus::Delayed),
            "soon" => Ok(EventStatus::Soon),
            _ => Err(()),
        }
    }
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
