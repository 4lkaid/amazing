use super::{account::AccountModel, action_type::ActionTypeModel};
use axum_kit::AppResult;
use serde::{Deserialize, Serialize};
use sqlx::{
    types::{chrono::NaiveDateTime, Decimal},
    PgExecutor,
};

#[derive(Deserialize, Serialize)]
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
    pub async fn create(
        executor: impl PgExecutor<'_>,
        account: &AccountModel,
        action_type: &ActionTypeModel,
        amount: f64,
        order_number: &str,
        description: &str,
    ) -> AppResult<()> {
        sqlx::query!(
            r#"insert into account_log (account_id, action_type_id, amount_available_balance, amount_frozen_balance, amount_total_income, amount_total_expense, available_balance_after, frozen_balance_after, total_income_after, total_expense_after, order_number, description)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"#,
            account.id,
            action_type.id,
            action_type.available_balance_change.calculate_change(amount),
            action_type.frozen_balance_change.calculate_change(amount),
            action_type.total_income_change.calculate_change(amount),
            action_type.total_expense_change.calculate_change(amount),
            account.available_balance,
            account.frozen_balance,
            account.total_income,
            account.total_expense,
            order_number,
            description,
        ).execute(executor).await?;
        Ok(())
    }
}
