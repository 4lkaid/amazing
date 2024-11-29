use crate::{model::action_type::ActionTypeModel, service::action_type::ActionTypeService};
use axum::Json;
use axum_kit::AppResult;

// 账户操作类型列表
pub async fn list() -> AppResult<Json<&'static Vec<ActionTypeModel>>> {
    let action_type = ActionTypeService::list();
    Ok(Json(action_type))
}
