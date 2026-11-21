mod choice;
mod population;
mod rechoose;
mod scenario;
mod stations;

use axum::{
    Json, Router,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use serde::Deserialize;
use std::{path::Path, sync::Arc};
use tower_http::cors::CorsLayer;

struct Core {
    latest: std::sync::Mutex<(u64, serde_json::Value)>,
    scenario_events: tokio::sync::watch::Sender<serde_json::Value>,
    scenario: scenario::Scenario,
}

#[derive(Deserialize)]
struct GeneralScenarioRequest {
    changes: Vec<serde_json::Value>,
}

impl Core {
    fn publish(&self, mut result: serde_json::Value) -> serde_json::Value {
        let mut latest = self.latest.lock().unwrap();
        result["revision"] = serde_json::json!(latest.0 + 1);
        *latest = (latest.0 + 1, result.clone());
        self.scenario_events.send_replace(result.clone());
        result
    }
}

async fn general_scenario(
    State(core): State<Arc<Core>>,
    Json(request): Json<GeneralScenarioRequest>,
) -> Json<serde_json::Value> {
    Json(core.publish(core.scenario.run(&request.changes)))
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
        latest: std::sync::Mutex::new((0, serde_json::Value::Null)),
        scenario_events: tokio::sync::watch::channel(serde_json::Value::Null).0,
        scenario: scenario::Scenario::load(Path::new(&directory)),
    });
    let app = Router::new()
        .route(
            "/api/scenario-latest",
            get(|State(core): State<Arc<Core>>| async move {
                let result = core.latest.lock().unwrap().1.clone();
                ([("cache-control", "no-store")], Json(result))
            }),
        )
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
