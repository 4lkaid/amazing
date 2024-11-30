use super::action_type::ActionTypeModel;
use axum_kit::AppResult;
use serde::{Deserialize, Serialize};
use sqlx::{
    types::{chrono::NaiveDateTime, Decimal},
    PgExecutor,
};

#[derive(Deserialize, Serialize)]
pub struct AccountModel {
    pub id: i32,
    pub user_id: i32,
    pub asset_type_id: i32,
    pub available_balance: Decimal,
    pub frozen_balance: Decimal,
    pub total_income: Decimal,
    pub total_expense: Decimal,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl AccountModel {
    pub async fn create(
        executor: impl PgExecutor<'_>,
        user_id: i32,
        asset_type_id: i32,
    ) -> AppResult<Self> {
        let account = sqlx::query_as!(
            Self,
            r#"insert into account (user_id, asset_type_id, is_active)
                values ($1, $2, true)
            returning
                id,
                user_id,
                asset_type_id,
                available_balance,
                frozen_balance,
                total_income,
                total_expense,
                is_active,
                created_at,
                updated_at"#,
            user_id,
            asset_type_id
        )
        .fetch_one(executor)
        .await?;
        Ok(account)
    }

    pub async fn find(
        executor: impl PgExecutor<'_>,
        user_id: i32,
        asset_type_id: i32,
    ) -> AppResult<Option<Self>> {
        let account = sqlx::query_as!(
            Self,
            r#"select
                id,
                user_id,
                asset_type_id,
                available_balance,
                frozen_balance,
                total_income,
                total_expense,
                is_active,
                created_at,
                updated_at
            from
                account
            where
                user_id = $1
                and asset_type_id = $2"#,
            user_id,
            asset_type_id
        )
        .fetch_optional(executor)
        .await?;
        Ok(account)
    }

    #[allow(dead_code)]
    pub async fn update_balance(
        executor: impl PgExecutor<'_>,
        user_id: i32,
        asset_type_id: i32,
        action_type: &ActionTypeModel,
        amount: f64,
    ) -> AppResult<Self> {
        let account = sqlx::query_as!(
            Self,
            r#"update account
                set available_balance = available_balance + $3,
                frozen_balance = frozen_balance + $4,
                total_income = total_income + $5,
                total_expense = total_expense + $6,
                updated_at = now()
            where
                user_id = $1
                and asset_type_id = $2
            returning
                id,
                user_id,
                asset_type_id,
                available_balance,
                frozen_balance,
                total_income,
                total_expense,
                is_active,
                created_at,
                updated_at"#,
            user_id,
            asset_type_id,
            action_type
                .available_balance_change
                .calculate_change(amount),
            action_type.frozen_balance_change.calculate_change(amount),
            action_type.total_income_change.calculate_change(amount),
            action_type.total_expense_change.calculate_change(amount),
        )
        .fetch_one(executor)
        .await?;
        Ok(account)
    }

    // 资产账户是否存在
    #[allow(dead_code)]
    pub async fn is_exists(
        executor: impl PgExecutor<'_>,
        user_id: i32,
        asset_type_id: i32,
    ) -> bool {
        if let Ok(Some(exists)) = sqlx::query_scalar!(
            r#"select exists(select 1 from account where user_id = $1 and asset_type_id = $2)"#,
            user_id,
            asset_type_id
        )
        .fetch_one(executor)
        .await
        {
            return exists;
        }
        false
    }

    // 资产账户是否启用
    #[allow(dead_code)]
    pub async fn is_active(
        executor: impl PgExecutor<'_>,
        user_id: i32,
        asset_type_id: i32,
    ) -> bool {
        if let Ok(Some(exists)) = sqlx::query_scalar!(
            r#"select exists(select 1 from account where user_id = $1 and asset_type_id = $2 and is_active = true)"#,
            user_id,
            asset_type_id
        )
        .fetch_one(executor)
        .await
        {
            return exists;
        }
        false
    }
}
