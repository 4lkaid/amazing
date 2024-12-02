use super::{action_type::ActionTypeService, asset_type::AssetTypeService};
use crate::{
    handler::account::{AccountActionRequest, AccountRequest},
    model::{account::AccountModel, account_log::AccountLogModel, action_type::Change},
};
use axum::http::StatusCode;
use axum_kit::{error::Error, postgres, AppResult};
use num_traits::cast::FromPrimitive;
use sqlx::types::Decimal;

pub struct AccountService;

impl<'a> AccountService {
    #[allow(dead_code)]
    pub async fn check_asset_type_id(asset_type_id: i32) -> AppResult<()> {
        if !AssetTypeService::is_active(asset_type_id) {
            return Err(Error::Custom(
                StatusCode::FORBIDDEN,
                "资产类型未启用".to_string(),
            ));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn check_action_type_id(action_type_id: i32) -> AppResult<()> {
        if !ActionTypeService::is_active(action_type_id) {
            return Err(Error::Custom(
                StatusCode::FORBIDDEN,
                "账户操作类型未启用".to_string(),
            ));
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn check_amount(amount: f64) -> AppResult<()> {
        if Decimal::from_f64(amount).unwrap().scale() > 6 {
            return Err(Error::Custom(
                StatusCode::FORBIDDEN,
                "无效值(最多6位小数)".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn create(account_request: &AccountRequest) -> AppResult<AccountModel> {
        Self::check_asset_type_id(account_request.asset_type_id).await?;
        let pool = postgres::conn();
        if AccountModel::is_exists(pool, account_request.user_id, account_request.asset_type_id)
            .await
        {
            return Err(Error::Custom(
                StatusCode::CONFLICT,
                "账户已存在".to_string(),
            ));
        }
        let account =
            AccountModel::create(pool, account_request.user_id, account_request.asset_type_id)
                .await?;
        Ok(account)
    }

    pub async fn info(account_request: &AccountRequest) -> AppResult<AccountModel> {
        Self::check_asset_type_id(account_request.asset_type_id).await?;
        let account = AccountModel::find(
            postgres::conn(),
            account_request.user_id,
            account_request.asset_type_id,
        )
        .await?;
        if let Some(account) = account {
            return Ok(account);
        }
        Err(Error::Custom(
            StatusCode::NOT_FOUND,
            "账户不存在".to_string(),
        ))
    }

    pub async fn actions(account_action_requests: &Vec<AccountActionRequest>) -> AppResult<()> {
        let mut tx = postgres::conn().begin().await?;
        for account_action_request in account_action_requests {
            Self::update_balance(&mut tx, account_action_request).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn update_balance(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        account_action_request: &AccountActionRequest,
    ) -> AppResult<()> {
        Self::check_asset_type_id(account_action_request.asset_type_id).await?;
        Self::check_action_type_id(account_action_request.action_type_id).await?;
        Self::check_amount(account_action_request.amount).await?;
        if !AccountModel::is_active(
            postgres::conn(),
            account_action_request.user_id,
            account_action_request.asset_type_id,
        )
        .await
        {
            return Err(Error::Custom(
                StatusCode::FORBIDDEN,
                "账户未启用".to_string(),
            ));
        }
        let amount = account_action_request.amount;
        let action_type = ActionTypeService::by_id(account_action_request.action_type_id).unwrap();
        let amount_available_balance = action_type
            .available_balance_change
            .calculate_change(amount);
        let amount_frozen_balance = action_type.frozen_balance_change.calculate_change(amount);
        let amount_total_income = action_type.total_income_change.calculate_change(amount);
        let amount_total_expense = action_type.total_expense_change.calculate_change(amount);
        let account = AccountModel::update_balance(
            &mut **tx,
            account_action_request.user_id,
            account_action_request.asset_type_id,
            amount_available_balance,
            amount_frozen_balance,
            amount_total_income,
            amount_total_expense,
        )
        .await?;
        // 扣减`可用余额/冻结余额`时，不允许`可用余额/冻结余额`为负数
        // 增加`可用余额/冻结余额`时，允许`可用余额/冻结余额`为负数
        // 因为管理员可能直接操作数据库修改用户`可用余额/冻结余额`，所以只在扣减操作才判断
        if (action_type.available_balance_change == Change::Dec
            && account.available_balance.is_sign_negative())
            || (action_type.frozen_balance_change == Change::Dec
                && account.frozen_balance.is_sign_negative())
        {
            return Err(Error::Custom(
                StatusCode::INTERNAL_SERVER_ERROR,
                "账户余额不足".to_string(),
            ));
        }
        AccountLogModel::create(
            &mut **tx,
            account.id,
            action_type.id,
            amount_available_balance,
            amount_frozen_balance,
            amount_total_income,
            amount_total_expense,
            account.available_balance,
            account.frozen_balance,
            account.total_income,
            account.total_expense,
            account_action_request
                .order_number
                .as_deref()
                .unwrap_or_default(),
            account_action_request
                .description
                .as_deref()
                .unwrap_or_default(),
        )
        .await?;
        Ok(())
    }
}
