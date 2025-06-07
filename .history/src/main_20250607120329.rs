#[macro_use] extern crate rocket;
use rocket::{serde::{json::Json, Serialize, Deserialize}};
use chrono::{DateTime, Utc};
use std::fs;
use std::str::FromStr;
use std::sync::Mutex;
use std::io::Write; 
use std::path::Path;
type SharedEvents = Mutex<Vec<EventDetail>>;



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
 

fn save_current_state(events: &Vec<EventDetail>) -> std::io::Result<()> {
    let serialized = serde_json::to_string_pretty(events)?;
    fs::write("curr_state.json", serialized)?;
    Ok(())
}

fn load_events_from_file(path: &str) -> Vec<EventDetail> {
    let data = fs::read_to_string(path).expect("Failed to read events.json");
    serde_json::from_str(&data).expect("Failed to parse JSON")  
}

fn load_initial_state() -> Vec<EventDetail> {
    // Try to load current state if it exists
    if let Ok(data) = fs::read_to_string("curr_state.json") {
        if let Ok(events) = serde_json::from_str(&data) {
            return events;
        }
    }

    let base_data = fs::read_to_string("events.json").expect("Failed to read events.json");
    let events: Vec<EventDetail> = serde_json::from_str(&base_data).expect("Invalid base JSON");

    fs::write("curr_state.json", serde_json::to_string_pretty(&events).unwrap())
        .expect("Failed to initialize curr_state.json");

    events
}


#[post("/api/v3/update/<event_name>/<status>")]
fn update_event(
    event_name: &str,
    status: &str,
    state: &rocket::State<SharedEvents>,
) -> Json<Vec<EventDetail>> {
    use std::str::FromStr;

    let mut events = state.lock().unwrap();

    let parsed_status = match EventStatus::from_str(&status.to_ascii_lowercase()) {
        Ok(s) => s,
        Err(_) => return Json(events.clone()),
    };

    for event in events.iter_mut() {
        if event.name == event_name {
            event.status = parsed_status.clone();
        }
    }

    fs::write("curr_state.json", serde_json::to_string_pretty(&*events).unwrap())
        .expect("Unable to write curr_state.json");

    Json(events.clone())
}


rate_limit! {
    "get_events" => [
        {
            quota: Quota::with_period(Duration::from_millis(5000)).unwrap(),
            filter: IpKeyFilter
        }
    ]
}


#[get("/api/v3/get/events")]
fn get_events(state: &rocket::State<SharedEvents>) -> Json<Vec<EventDetail>> {
    let events = state.lock().unwrap();
    Json(events.clone())
}

#[launch]
fn rocket() -> _ {
    let events = load_initial_state();
    rocket::build()
        .manage(Mutex::new(events))
        .mount("/", routes![
            update_event,
            get_events
        ])
}

