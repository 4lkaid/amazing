use crate::{
    model::account::AccountModel,
    service::{
        account::AccountService, action_type::ActionTypeService, asset_type::AssetTypeService,
    },
};
use axum::{http::StatusCode, Json};
use axum_kit::{validation::ValidatedJson, AppResult};
use num_traits::cast::FromPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::types::Decimal;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Validate, Debug, Serialize)]
pub struct AccountRequest {
    #[validate(range(min = 1))]
    pub user_id: i32,
    #[validate(custom(function = "validate_asset_type_id"))]
    pub asset_type_id: i32,
}

#[derive(Deserialize, Validate, Debug, Serialize)]
pub struct AccountActionRequest {
    #[validate(range(min = 1))]
    pub user_id: i32,
    #[validate(custom(function = "validate_asset_type_id"))]
    pub asset_type_id: i32,
    #[validate(custom(function = "validate_action_type_id"))]
    pub action_type_id: i32,
    #[validate(range(min = 0.000001), custom(function = "validate_amount"))]
    pub amount: f64,
    #[validate(length(min = 32))]
    pub order_number: String,
    #[validate(length(min = 1))]
    pub description: String,
}

fn validate_asset_type_id(id: i32) -> Result<(), ValidationError> {
    if !AssetTypeService::is_active(id) {
        return Err(ValidationError::new("无效值"));
    }
    Ok(())
}

fn validate_action_type_id(id: i32) -> Result<(), ValidationError> {
    if !ActionTypeService::is_active(id) {
        return Err(ValidationError::new("无效值"));
    }
    Ok(())
}

fn validate_amount(amount: f64) -> Result<(), ValidationError> {
    if Decimal::from_f64(amount).unwrap().scale() > 6 {
        return Err(ValidationError::new("无效值(最多6位小数)"));
    }
    Ok(())
}

// 添加账户
pub async fn create(
    ValidatedJson(payload): ValidatedJson<AccountRequest>,
) -> AppResult<(StatusCode, Json<AccountModel>)> {
    let account = AccountService::create(&payload).await?;
    Ok((StatusCode::CREATED, Json(account)))
}

// 账户信息
pub async fn info(
    ValidatedJson(payload): ValidatedJson<AccountRequest>,
) -> AppResult<Json<AccountModel>> {
    let account = AccountService::info(&payload).await?;
    Ok(Json(account))
}

// 账户操作
// 仅涉及可用余额、冻结余额、累计收入、累计支出的变更
pub async fn actions(
    ValidatedJson(payload): ValidatedJson<Vec<AccountActionRequest>>,
) -> AppResult<()> {
    AccountService::actions(&payload).await
}
