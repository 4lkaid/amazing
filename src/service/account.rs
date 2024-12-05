use super::action_type::ActionTypeService;
use crate::{
    handler::account::{AccountActionRequest, AccountRequest},
    model::{
        account::AccountModel,
        account_log::AccountLogModel,
        action_type::{ActionTypeModel, Change},
    },
};
use axum::http::StatusCode;
use axum_kit::{error::Error, postgres, AppResult};
use num_traits::FromPrimitive;
use sqlx::types::Decimal;
use validator::Validate;

pub struct AccountService;

impl AccountService {
    pub async fn check_account_is_active(user_id: i32, asset_type_id: i32) -> AppResult<()> {
        if !AccountModel::is_active(postgres::conn(), user_id, asset_type_id).await {
            return Err(Error::Custom(
                StatusCode::FORBIDDEN,
                "账户未启用".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn check_balance_before_update(
        action_type: &ActionTypeModel,
        account: &AccountModel,
        amount: Decimal,
    ) -> AppResult<()> {
        if (action_type.available_balance_change == Change::Dec
            && account.available_balance < amount)
            || (action_type.frozen_balance_change == Change::Dec && account.frozen_balance < amount)
        {
            return Err(Error::Custom(
                StatusCode::PAYMENT_REQUIRED,
                "账户余额不足".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn check_balance_after_update(
        action_type: &ActionTypeModel,
        account: &AccountModel,
    ) -> AppResult<()> {
        if (action_type.available_balance_change == Change::Dec
            && account.available_balance.is_sign_negative())
            || (action_type.frozen_balance_change == Change::Dec
                && account.frozen_balance.is_sign_negative())
        {
            return Err(Error::Custom(
                StatusCode::PAYMENT_REQUIRED,
                "账户余额不足".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn check_account_log_exists(
        account_id: i32,
        action_type_id: i32,
        order_number: &str,
    ) -> AppResult<()> {
        if AccountLogModel::is_exists(postgres::conn(), account_id, action_type_id, order_number)
            .await
        {
            return Err(Error::Custom(
                StatusCode::CONFLICT,
                "该订单号已处理".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn create(account_request: &AccountRequest) -> AppResult<AccountModel> {
        account_request.validate()?;
        let pool = postgres::conn();
        let account =
            AccountModel::create(pool, account_request.user_id, account_request.asset_type_id)
                .await?;
        Ok(account)
    }

    pub async fn info(account_request: &AccountRequest) -> AppResult<AccountModel> {
        account_request.validate()?;
        let account = AccountModel::find(
            postgres::conn(),
            account_request.user_id,
            account_request.asset_type_id,
        )
        .await?;
        Ok(account)
    }

    pub async fn actions(account_action_requests: &Vec<AccountActionRequest>) -> AppResult<()> {
        account_action_requests.validate()?;
        // 开启事务前检查账户状态、余额是否充足以及订单号是否已处理，从而避免不必要的数据库操作开销
        for account_action_request in account_action_requests {
            let action_type =
                ActionTypeService::by_id(account_action_request.action_type_id).unwrap();
            let account = AccountModel::find(
                postgres::conn(),
                account_action_request.user_id,
                account_action_request.asset_type_id,
            )
            .await?;
            if !account.is_active {
                return Err(Error::Custom(
                    StatusCode::FORBIDDEN,
                    "账户未启用".to_string(),
                ));
            }
            // 操作前检查余额是否充足
            Self::check_balance_before_update(
                action_type,
                &account,
                Decimal::from_f64(account_action_request.amount.abs())
                    .unwrap()
                    .trunc_with_scale(6),
            )
            .await?;
            Self::check_account_log_exists(
                account.id,
                action_type.id,
                account_action_request.order_number.as_str(),
            )
            .await?;
        }
        let mut tx = postgres::conn().begin().await?;
        for account_action_request in account_action_requests {
            Self::update_balance(&mut tx, account_action_request).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn update_balance(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        account_action_request: &AccountActionRequest,
    ) -> AppResult<()> {
        // 存在外部校验时间过长的可能，需要重新校验账户状态
        Self::check_account_is_active(
            account_action_request.user_id,
            account_action_request.asset_type_id,
        )
        .await?;
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
        Self::check_balance_after_update(action_type, &account).await?;
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
            account_action_request.order_number.as_ref(),
            account_action_request.description.as_ref(),
        )
        .await?;
        Ok(())
    }
}
