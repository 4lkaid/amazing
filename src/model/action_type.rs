use axum_kit::AppResult;
use num_traits::cast::FromPrimitive;
use serde::Serialize;
use sqlx::{
    types::{chrono::NaiveDateTime, Decimal},
    PgExecutor,
};

#[derive(Serialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "change_enum", rename_all = "UPPERCASE")]
pub enum Change {
    Inc,
    Dec,
    None,
}

impl Change {
    #[allow(dead_code)]
    pub fn calculate_change(&self, amount: f64) -> Decimal {
        let decimal_amount = Decimal::from_f64(amount.abs()).unwrap().trunc_with_scale(6);
        match self {
            Change::Inc => decimal_amount,
            Change::Dec => -decimal_amount,
            Change::None => Decimal::ZERO,
        }
    }
}

#[derive(Serialize)]
pub struct ActionTypeModel {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub available_balance_change: Change,
    pub frozen_balance_change: Change,
    pub total_income_change: Change,
    pub total_expense_change: Change,
    #[allow(dead_code)]
    #[serde(skip_serializing)]
    pub is_active: bool,
    #[allow(dead_code)]
    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,
    #[allow(dead_code)]
    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}

impl ActionTypeModel {
    pub async fn fetch_all(executor: impl PgExecutor<'_>) -> AppResult<Vec<Self>> {
        let action_types: Vec<Self> = sqlx::query_as!(
            Self,
            r#"select
                id,
                name,
                description,
                available_balance_change as "available_balance_change!: Change",
                frozen_balance_change as "frozen_balance_change!: Change",
                total_income_change as "total_income_change!: Change",
                total_expense_change as "total_expense_change!: Change",
                is_active,
                created_at,
                updated_at
            from
                action_type
            where
                is_active = true"#
        )
        .fetch_all(executor)
        .await?;
        Ok(action_types)
    }
}
