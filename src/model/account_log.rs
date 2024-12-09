use axum_kit::AppResult;
use serde::Serialize;
use sqlx::{
    types::{chrono::NaiveDateTime, Decimal},
    PgExecutor,
};

#[derive(Serialize)]
pub struct AccountLogModel {
    pub id: i64,
    pub account_id: i32,
    pub action_type_id: i32,
    pub amount_available_balance: Decimal,
    pub amount_frozen_balance: Decimal,
    pub amount_total_income: Decimal,
    pub amount_total_expense: Decimal,
    pub available_balance_after: Decimal,
    pub frozen_balance_after: Decimal,
    pub total_income_after: Decimal,
    pub total_expense_after: Decimal,
    pub order_number: String,
    pub description: String,
    pub created_at: NaiveDateTime,
}

impl AccountLogModel {
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        executor: impl PgExecutor<'_>,
        account_id: i32,
        action_type_id: i32,
        amount_available_balance: Decimal,
        amount_frozen_balance: Decimal,
        amount_total_income: Decimal,
        amount_total_expense: Decimal,
        available_balance_after: Decimal,
        frozen_balance_after: Decimal,
        total_income_after: Decimal,
        total_expense_after: Decimal,
        order_number: &str,
        description: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"insert into account_log (
                account_id,
                action_type_id,
                amount_available_balance,
                amount_frozen_balance,
                amount_total_income,
                amount_total_expense,
                available_balance_after,
                frozen_balance_after,
                total_income_after,
                total_expense_after,
                order_number,
                description
            )
            values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
            account_id,
            action_type_id,
            amount_available_balance,
            amount_frozen_balance,
            amount_total_income,
            amount_total_expense,
            available_balance_after,
            frozen_balance_after,
            total_income_after,
            total_expense_after,
            order_number,
            description
        )
        .execute(executor)
        .await?;
        Ok(())
    }

    // 账户操作日志是否存在
    #[allow(dead_code)]
    pub async fn is_exists(
        executor: impl PgExecutor<'_>,
        account_id: i32,
        action_type_id: i32,
        order_number: &str,
    ) -> bool {
        if let Ok(Some(exists)) = sqlx::query_scalar!(
            r#"select exists(select 1 from account_log where account_id = $1 and action_type_id = $2 and order_number = $3)"#,
            account_id,
            action_type_id,
            order_number
        )
        .fetch_one(executor)
        .await
        {
            return exists;
        }
        false
    }
}
