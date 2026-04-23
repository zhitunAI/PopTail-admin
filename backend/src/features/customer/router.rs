use axum::Router;
use axum::routing::get;

use crate::state::AppState;

use super::{customer_create, customer_delete, customer_get, customer_list, customer_update};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/customer/customerList", get(customer_list))
        .route(
            "/customer/customer",
            get(customer_get)
                .post(customer_create)
                .put(customer_update)
                .delete(customer_delete),
        )
}
