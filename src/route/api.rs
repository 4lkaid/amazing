use crate::handler;
use axum::{middleware, routing::get, Router};
use axum_kit::middleware::{cors, request_id, request_response_logger, trace};
use tower::ServiceBuilder;

pub fn init() -> Router {
    Router::new()
        // 获取资产类型
        .route("/assets", get(handler::asset_type::list))
        // 获取账户操作类型
        .route("/actions", get(handler::action_type::list))
        .layer(
            ServiceBuilder::new()
                .layer(request_id::set_request_id())
                .layer(request_id::propagate_request_id())
                .layer(trace::trace())
                .layer(cors::cors())
                .layer(middleware::from_fn(
                    request_response_logger::print_request_response,
                )),
        )
}
