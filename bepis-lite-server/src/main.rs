use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
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
    #[serde(default)]
    quantity: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Order {
    items: Vec<Item>,
    total_cost: f64,
}

impl Order {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            total_cost: 0.0,
        }
    }

    fn new_with_item(item: Item) -> Self {
        let mut order = Self::new();
        order.add_item(item);
        order
    }

    fn add_item(&mut self, item: Item) {
        self.total_cost += item.price;
        self.items.push(item);
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Item {
    name: String,
    description: String,
    price: f64,
    category: String,
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
        .route("/calls/:id/order/items", post(add_items_to_order))
        .route("/calls/:id/order/items", delete(remove_item_from_order))
        .route("/calls/:id/order", delete(clear_order))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
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
) -> Result<Json<Call>, (StatusCode, String)> {
    dbg!("getting info for a call");
    if let Ok(state) = state.lock() {
        if let Some(call) = state.calls.get(&id) {
            dbg!(&call.order);
            return Ok(Json(call.clone()));
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                "Bad Request - call does not exist.".to_string(),
            ));
        }
    }

    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".to_string(),
    ))
}

#[debug_handler]
async fn get_order(
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<Option<Order>>, (StatusCode, String)> {
    dbg!("getting info for a call order");
    if let Ok(state) = state.lock() {
        if let Some(call) = state.calls.get(&id) {
            dbg!(&call.order);
            return Ok(Json(call.order.clone()));
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                "Bad Request - call does not exist.".to_string(),
            ));
        }
    }

    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Internal Server Error".to_string(),
    ))
}

#[debug_handler]
async fn add_items_to_order(
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<OrderRequest>,
) -> String {
    dbg!("updating order (adding item)");
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

            let quantity = payload.quantity.unwrap_or(1);

            if let Some(order) = &mut call.order {
                for _ in 0..quantity {
                    order.add_item(new_item.clone());
                }
            } else {
                call.order = Some(Order::new_with_item(new_item));
            }
        }

        format!("We were able to successfully add the item(s) to the order! The current state of the order is: {:?}", call.order)
    } else {
        "We were unable to add the item to the order as the specified call id does not exist."
            .to_string()
    }
}

#[debug_handler]
async fn remove_item_from_order(
    Path(id): Path<uuid::Uuid>,
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<OrderRequest>,
) -> String {
    dbg!("updating order (removing item)");
    dbg!(&payload);
    let mut state = state.lock().expect("failed to obtain state lock");
    if state.calls.contains_key(&id) {
        let call = state.calls.get_mut(&id).expect("failed to obtain the call");

        if let Some(order) = &mut call.order {
            let quantity = payload.quantity.unwrap_or(1);

            for _ in 0..quantity {
                let index = order
                    .items
                    .iter()
                    .position(|item| *item.name == payload.item);
                if let Some(index) = index {
                    let removed_item = order.items.remove(index);
                    order.total_cost -= removed_item.price;
                }
            }
        }
        format!("We were able to successfully remove the item(s) from the order if they were present! The current state of the order is: {:?}", call.order)
    } else {
        "We were unable to remove the item from the order as the specified call id does not exist."
            .to_string()
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
    "We were able to successfully cleared the call's order!"
}
