use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Json, Path, State};
use axum::routing::{delete, get, post};
use axum::{debug_handler, Router};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Call {
    id: uuid::Uuid,
    order: Option<Order>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct OrderRequest {
    item: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Order {
    items: Vec<Item>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Item {
    name: String,
    description: String,
    price: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Menu {
    items: HashMap<String, Item>,
}

struct AppState {
    calls: HashMap<uuid::Uuid, Call>,
    menu: Menu,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(AppState {
        calls: HashMap::new(),
        menu: Menu {
            items: HashMap::new(),
        },
    }));

    // Note: this isn't super REST-y
    let app = Router::new()
        .route("/menu", get(get_menu))
        .route("/menu/items", post(add_item_to_menu))
        .route("/menu/items", delete(clear_menu))
        .route("/calls", post(create_call))
        .route("/calls/:id", get(get_call))
        .route("/calls/:id/order", get(get_order))
        .route("/calls/:id/order/items", post(update_order))
        .route("/calls/:id/order", delete(clear_order))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn get_menu(State(state): State<Arc<Mutex<AppState>>>) -> Json<Menu> {
    dbg!("getting info for the menu");
    let state = state.lock().expect("failed to obtain state lock");
    dbg!(&state.menu);
    Json(state.menu.clone())
}

#[debug_handler]
async fn add_item_to_menu(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<Item>,
) -> &'static str {
    dbg!("adding item to menu");
    dbg!(&payload);
    let mut state = state.lock().expect("failed to obtain state lock");
    (*state).menu.items.insert(payload.name.clone(), payload);
    "successfully added item to menu"
}

#[debug_handler]
async fn clear_menu(State(state): State<Arc<Mutex<AppState>>>) -> &'static str {
    dbg!("clearing menu");
    let mut state = state.lock().expect("failed to obtain state lock");
    (*state).menu.items.clear();
    "successfully cleared the menu"
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
    Json(payload): Json<OrderRequest>,
) -> String {
    dbg!("updating order");
    dbg!(&payload);
    let mut state = state.lock().expect("failed to obtain state lock");
    let menu = state.menu.clone();
    if state.calls.contains_key(&id) {
        let call = state.calls.get_mut(&id).expect("failed to obtain the call");
        let item = payload.item;
        if !menu.items.contains_key(&item) {
            return "We were unable to submit this order as there were items requested that were not on the menu.".to_string();
        } else {
            let new_item = menu
                .items
                .get(&item)
                .expect("Failed to get item from the menu.")
                .clone();

            if let Some(order) = &mut call.order {
                order.items.push(new_item);
            } else {
                call.order = Some(Order {
                    items: vec![new_item],
                });
            }
        }

        format!(
            "We were able to successfully submit the order! The total price for the order is $10",
        )
    } else {
        "We were unable to submit the order as the specified call id does not exist.".to_string()
    }
}

#[debug_handler]
async fn clear_order(
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> &'static str {
    dbg!("clearing a call order");
    let mut state = state.lock().expect("failed to obtain state lock");
    let call = state.calls.get_mut(&id).expect("failed to obtain the call");
    dbg!(&call.order);
    call.order = None;
    "successfully cleared the call's order"
}
