#[macro_use] extern crate rocket;

use rocket::{serde::{json::Json, Serialize, Deserialize}};
use chrono::{DateTime, Utc};
use std::{fs, str::FromStr, sync::Mutex, collections::HashMap};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use dotenvy::{dotenv, var};
use rocket_governor::{Method, Quota, RocketGovernable, RocketGovernor};
use rocket_cors::{CorsOptions};
use rocket_cors::{AllowedOrigins, AllowedHeaders};
type SharedEvents = Mutex<Vec<EventDetail>>;

pub struct ApiKey(String);

struct ApiKeys {
    root_key: String,
    event_keys: HashMap<String, String>, // event_name -> api_key
}

impl ApiKeys {
    fn load_from_env() -> Self {
        dotenv().ok();

        let root_key = var("API_SECRET_KEY").expect("API_SECRET_KEY not set");

        let mut event_keys = HashMap::new();

        if let Ok(key) = var("YUKTI_API_KEY") {
            event_keys.insert("Yukti".to_string(), key);
        }
        if let Ok(key) = var("NATYA_API_KEY") {
            event_keys.insert("Natya-Sutra".to_string(), key);
        }
        if let Ok(key) = var("NAADA_API_KEY") {
            event_keys.insert("Naada-Nirvana".to_string(), key);
        }
        if let Ok(key) = var("NAZAKAT_API_KEY") {
            event_keys.insert("Nazakat".to_string(), key);
        }
        if let Ok(key) = var("NATAKA_API_KEY") {
            event_keys.insert("Nataka".to_string(), key);
        }
        // Add more events + keys as needed

        ApiKeys { root_key, event_keys }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let api_keys = match req.rocket().state::<ApiKeys>() {
            Some(state) => state,
            None => return Outcome::Error((Status::InternalServerError, ())),
        };

        // Extract Authorization: Bearer <key>
        let key_opt = req.headers()
            .get_one("Authorization")
            .and_then(|header| header.strip_prefix("Bearer "));

        match key_opt {
            Some(key) => Outcome::Success(ApiKey(key.to_string())),
            None => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}

pub struct RateLimitGuard;

impl<'r> RocketGovernable<'r> for RateLimitGuard {
    fn quota(_method: Method, _route_name: &str) -> Quota {
        Quota::per_second(Self::nonzero(1u32))
    }
}

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
    api_key: ApiKey,
    api_keys: &rocket::State<ApiKeys>,
    _limitguard: RocketGovernor<RateLimitGuard>
) -> Result<Json<Vec<EventDetail>>, Status> {

    // Root key can update any event
    if api_key.0 != api_keys.root_key {
        // Otherwise check if key matches the event's allowed key
        match api_keys.event_keys.get(event_name) {
            Some(expected_key) if *expected_key == api_key.0 => (),
            _ => return Err(Status::Forbidden),
        }
    }

    let mut events = state.lock().unwrap();

    let parsed_status = match EventStatus::from_str(&status.to_ascii_lowercase()) {
        Ok(s) => s,
        Err(_) => return Ok(Json(events.clone())),
    };

    let mut updated = false;
    for event in events.iter_mut() {
        if event.name == event_name {
            event.status = parsed_status.clone();
            updated = true;
        }
    }

    if updated {
        fs::write("curr_state.json", serde_json::to_string_pretty(&*events).unwrap())
            .expect("Unable to write curr_state.json");
    }

    Ok(Json(events.clone()))
}

#[get("/api/v3/get/events")]
fn get_events(
    state: &rocket::State<SharedEvents>,
    _limitguard: RocketGovernor<RateLimitGuard>
) -> Json<Vec<EventDetail>> {
    let events = state.lock().unwrap();
    Json(events.clone())
}

#[launch]
fn rocket() -> _ {
    let events = load_initial_state();
    let api_keys = ApiKeys::load_from_env();

    let allowed_origins = AllowedOrigins::some_exact(&["https://adharvaa.com"]);

    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec!["GET".parse().unwrap(), "POST".parse().unwrap()]
            .into_iter()
            .collect(),
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Content-Type",
            "Origin",
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error creating CORS fairing");
    rocket::build()
        .manage(Mutex::new(events))
        .manage(api_keys)
        .mount("/", routes![
            update_event,
            get_events
        ])
        .attach(cors)
}
