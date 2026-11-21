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
    half_hour: u8,
}

fn next_revision(snapshot: &Value) -> u64 {
    snapshot["revision"].as_u64().unwrap_or(0) + 1
}

async fn general_scenario(
    State(core): State<Arc<Core>>,
    Json(request): Json<GeneralScenarioRequest>,
) -> Json<Value> {
    let mut result = core.scenario.run(&request.changes);
    core.scenario_events.send_modify(|latest| {
        result["revision"] = json!(next_revision(latest));
        if let Some(scene) = latest.get("scene") {
            result["scene"] = scene.clone();
        }
        *latest = result.clone();
    });
    Json(result)
}

async fn set_scene(
    State(core): State<Arc<Core>>,
    Json(request): Json<SceneRequest>,
) -> Result<Json<Value>, (StatusCode, &'static str)> {
    let place = request.place;
    let named_place = matches!(place.kind.as_str(), "station" | "place" | "comuna" | "line")
        && place.id.as_ref().is_some_and(|id| !id.is_empty());
    if place.kind != "point" && !named_place {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Use a point or a station, place, comuna or line with its catalog id.",
        ));
    }
    if request.half_hour >= 48
        || !(-180.0..=180.0).contains(&place.longitude)
        || !(-90.0..=90.0).contains(&place.latitude)
    {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "Use WGS84 coordinates and a half_hour from 0 to 47.",
        ));
    }

    let needs_base = core.scenario_events.borrow().is_null();
    let baseline = needs_base.then(|| core.scenario.run(&[]));
    let mut response = Value::Null;
    core.scenario_events.send_modify(|latest| {
        let revision = next_revision(latest);
        if latest.is_null() {
            *latest = baseline.unwrap();
        }
        let scene = json!({"revision": revision, "place": place, "half_hour": request.half_hour});
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
        .route("/api/scene", post(set_scene))
        .route("/api/scenario-events", get(scenario_events))
        .route("/api/scenario", post(general_scenario))
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
