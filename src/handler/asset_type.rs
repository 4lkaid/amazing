use crate::{model::asset_type::AssetTypeModel, service::asset_type::AssetTypeService};
use axum::Json;
use axum_kit::AppResult;

// 资产类型列表
pub async fn list() -> AppResult<Json<&'static Vec<AssetTypeModel>>> {
    let asset_type = AssetTypeService::list();
    Ok(Json(asset_type))
}
