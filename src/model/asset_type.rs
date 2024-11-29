use axum_kit::AppResult;
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono::NaiveDateTime, PgPool};

#[derive(Deserialize, Serialize)]
pub struct AssetTypeModel {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl AssetTypeModel {
    pub async fn fetch_all(pool: &PgPool) -> AppResult<Vec<Self>> {
        let asset_types: Vec<Self> = sqlx::query_as!(
            Self,
            r#"select
                id,
                name,
                description,
                is_active,
                created_at,
                updated_at
            from
                asset_type
            where
                is_active = true"#
        )
        .fetch_all(pool)
        .await?;
        Ok(asset_types)
    }
}
