mod choice;
mod population;
mod rechoose;
mod scenario;
mod stations;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use santiago_scenario_core::activity::Activity;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc};
use tower_http::cors::CorsLayer;

struct Core {
    scenario_events: tokio::sync::watch::Sender<Value>,
    scenario: scenario::Scenario,
}

#[derive(Deserialize)]
struct GeneralScenarioRequest {
    changes: Vec<serde_json::Value>,
    #[serde(default)]
    require_all: bool,
    scene: Option<SceneRequest>,
    activity: Option<ScenarioActivity>,
}

#[derive(Deserialize)]
struct ScenarioActivity {
    #[serde(flatten)]
    model: Activity,
    mode: Option<String>,
    motor_access: Option<Value>,
    #[serde(default)]
    geometry: Vec<ActivityGeometry>,
}

#[derive(Deserialize, Serialize)]
struct ActivityGeometry {
    id: String,
    #[serde(alias = "coordinates")]
    geometry: Vec<[f64; 2]>,
}

impl ScenarioActivity {
    fn result(&self) -> Result<Value, &'static str> {
        let mut result = json!(self.model.run()?);
        result["segments"] = json!(self.geometry);
        if let Some(mode) = &self.mode {
            result["mode"] = json!(mode);
        }
        if let Some(access) = &self.motor_access {
            result["motor_access"] = access.clone();
        }
        Ok(result)
    }
}

#[derive(Deserialize, Serialize)]
struct ScenePlace {
    #[serde(rename = "type")]
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    longitude: f64,
    latitude: f64,
}

#[derive(Deserialize)]
struct SceneRequest {
    place: ScenePlace,
    half_hour: Option<u8>,
}

impl SceneRequest {
    fn validate(&self) -> Result<(), (StatusCode, &'static str)> {
        let place = &self.place;
        let named_place = matches!(place.kind.as_str(), "station" | "place" | "comuna" | "line")
            && place.id.as_ref().is_some_and(|id| !id.is_empty());
        if place.kind != "point" && !named_place {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Use a point or a station, place, comuna or line with its catalog id.",
            ));
        }
        if self.half_hour.is_some_and(|bin| bin >= 48)
            || !(-180.0..=180.0).contains(&place.longitude)
            || !(-90.0..=90.0).contains(&place.latitude)
        {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Use WGS84 coordinates and a half_hour from 0 to 47.",
            ));
        }
        Ok(())
    }

    fn value(&self, revision: u64) -> Value {
        let mut scene = json!({"revision": revision, "place": self.place});
        if let Some(bin) = self.half_hour {
            scene["half_hour"] = json!(bin);
        }
        scene
    }
}

async fn building_footprint(
    State(core): State<Arc<Core>>,
    Json(locator): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    core.scenario.mapped_footprint(&locator)
        .map(Json)
        .map_err(|reason| (StatusCode::UNPROCESSABLE_ENTITY, reason))
}

fn next_revision(snapshot: &Value) -> u64 {
    snapshot["revision"].as_u64().unwrap_or(0) + 1
}

async fn general_scenario(
    State(core): State<Arc<Core>>,
    Json(request): Json<GeneralScenarioRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    if let Some(scene) = &request.scene {
        scene.validate().map_err(|(status, reason)| (status, reason.to_owned()))?;
    }
    let activity = request.activity.as_ref()
        .map(ScenarioActivity::result)
        .transpose()
        .map_err(|reason| (StatusCode::UNPROCESSABLE_ENTITY, reason.to_owned()))?;
    let mut result = core.scenario.run(&request.changes);
    let rejected = result["not_modeled"].as_array().unwrap();
    if (request.require_all || activity.is_some()) && !rejected.is_empty() {
        let reasons = rejected.iter().filter_map(|edit| edit["reason"].as_str()).collect::<Vec<_>>();
        let change = if activity.is_some() { "activity" } else { "placement" };
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("The {change} was not applied. {}", reasons.join(" ")),
        ));
    }
    if let Some(activity) = activity {
        result["activity"] = activity;
    }
    core.scenario_events.send_modify(|latest| {
        let revision = next_revision(latest);
        result["revision"] = json!(revision);
        if let Some(activity) = result.get_mut("activity") {
            activity["playback_revision"] = json!(revision);
        }
        if let Some(scene) = request.scene {
            result["scene"] = scene.value(revision);
        } else if let Some(scene) = latest.get("scene") {
            result["scene"] = scene.clone();
        }
        *latest = result.clone();
    });
    Ok(Json(result))
}

// A cached replay: publish a saved engine result to the viewer without a new run.
async fn play_replay(State(core): State<Arc<Core>>, Json(mut result): Json<Value>) -> Json<Value> {
    core.scenario_events.send_modify(|latest| {
        let revision = json!(next_revision(latest));
        result["revision"] = revision.clone();
        // The viewer flies the camera on a new scene revision.
        result["scene"]["revision"] = revision.clone();
        if let Some(activity) = result.get_mut("activity").and_then(Value::as_object_mut) {
            activity.insert("playback_revision".into(), revision);
        }
        *latest = result.clone();
    });
    Json(json!({"revision": result["revision"], "scene": result["scene"]}))
}

async fn set_scene(
    State(core): State<Arc<Core>>,
    Json(request): Json<SceneRequest>,
) -> Result<Json<Value>, (StatusCode, &'static str)> {
    request.validate()?;
    let needs_base = core.scenario_events.borrow().is_null();
    let baseline = needs_base.then(|| core.scenario.run(&[]));
    let mut response = Value::Null;
    core.scenario_events.send_modify(|latest| {
        let revision = next_revision(latest);
        if latest.is_null() {
            *latest = baseline.unwrap();
        }
        let scene = request.value(revision);
        latest["scene"] = scene.clone();
        latest["revision"] = json!(revision);
        response = json!({"revision": revision, "scene": scene});
    });
    Ok(Json(response))
}

async fn scenario_events(
    State(core): State<Arc<Core>>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let mut receiver = core.scenario_events.subscribe();
    let initial = receiver.borrow_and_update().clone();
    let stream = futures_util::stream::unfold(
        (receiver, Some(initial)),
        |(mut receiver, initial)| async move {
            let result = match initial.filter(|result| !result.is_null()) {
                Some(result) => result,
                None => {
                    receiver.changed().await.ok()?;
                    receiver.borrow_and_update().clone()
                }
            };
            let event = Event::default()
                .event("scenario")
                .id(result["revision"].to_string())
                .json_data(result)
                .unwrap();
            Some((Ok(event), (receiver, None)))
        },
    );
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let directory = args.next().unwrap_or_else(|| "data/mvp".to_owned());
    let port = args.next().unwrap_or_else(|| "8787".to_owned());
    let core = Arc::new(Core {
        scenario_events: tokio::sync::watch::channel(serde_json::Value::Null).0,
        scenario: scenario::Scenario::load(Path::new(&directory)),
    });
    let app = Router::new()
        .route(
            "/api/scenario-latest",
            get(|State(core): State<Arc<Core>>| async move {
                let result = core.scenario_events.borrow().clone();
                ([("cache-control", "no-store")], Json(result))
            }),
        )
        .route("/api/building-footprint", post(building_footprint))
        .route("/api/scene", post(set_scene))
        .route("/api/scenario-events", get(scenario_events))
        .route("/api/scenario", post(general_scenario))
        // A saved result (4.1 MB measured) is larger than the default 2 MB body limit.
        .route("/api/replay", post(play_replay).layer(axum::extract::DefaultBodyLimit::max(64 << 20)))
        .layer(CorsLayer::permissive())
        .with_state(core);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    eprintln!(
        "Scenario core listening on http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}
