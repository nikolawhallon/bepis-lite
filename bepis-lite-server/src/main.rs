use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Json, Path, State};
use axum::routing::{get, post};
use axum::{debug_handler, Router};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Call {
    id: uuid::Uuid,
    order: Option<Order>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Order {
    item: Item,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Item {
    Pepsi,
    Coke,
}

struct AppState {
    calls: HashMap<uuid::Uuid, Call>,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(AppState {
        calls: HashMap::new(),
    }));

    // Note: this isn't super REST-y
    let app = Router::new()
        .route("/calls", post(create_call))
        .route("/calls/:id", get(get_call))
        .route("/calls/:id/order", get(get_order))
        .route("/calls/:id/order", post(update_order))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_call(State(state): State<Arc<Mutex<AppState>>>) -> String {
    dbg!("creating a call");
    let id = uuid::Uuid::new_v4();
    let mut state = state.lock().expect("failed to obtain state lock");
    state.calls.insert(id, Call { id, order: None });
    dbg!(&id);
    id.to_string()
}

#[debug_handler]
async fn get_call(
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<Call> {
    dbg!("getting info for a call");
    let state = state.lock().expect("failed to obtain state lock");
    let call = state.calls.get(&id).expect("failed to obtain the call");
    dbg!(&call);
    Json(call.clone())
}

#[debug_handler]
async fn get_order(
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<Option<Order>> {
    dbg!("getting info for a call order");
    let state = state.lock().expect("failed to obtain state lock");
    let call = state.calls.get(&id).expect("failed to obtain the call");
    dbg!(&call.order);
    Json(call.order.clone())
}

#[debug_handler]
async fn update_order(
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<Order>,
) -> &'static str {
    dbg!("updating order");
    dbg!(&payload);
    let mut state = state.lock().expect("failed to obtain state lock");
    if state.calls.contains_key(&id) {
        let call = state.calls.get_mut(&id).expect("failed to obtain the call");
        call.order = Some(payload);

        "We were able to successfully submit the order!"
    } else {
        "We were unable to submit the order as the specified call id does not exist."
    }
}
