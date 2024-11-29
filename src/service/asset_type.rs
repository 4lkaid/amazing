use crate::model::asset_type::AssetTypeModel;
use axum_kit::{postgres, AppResult};
use std::sync::OnceLock;

static ASSET_TYPE: OnceLock<Vec<AssetTypeModel>> = OnceLock::new();

pub struct AssetTypeService;
impl AssetTypeService {
    pub async fn init() -> AppResult<()> {
        let asset_types = AssetTypeModel::fetch_all(postgres::conn()).await?;
        let _ = ASSET_TYPE
            .set(asset_types)
            .map_err(|_| "Failed to initialize ASSET_TYPE");
        Ok(())
    }

    pub fn list() -> &'static Vec<AssetTypeModel> {
        ASSET_TYPE.get().expect("ASSET_TYPE is not initialized")
    }

    #[allow(dead_code)]
    pub fn is_active(id: i32) -> bool {
        let asset_types = Self::list();
        asset_types.iter().any(|asset_type| asset_type.id == id)
    }

    #[allow(dead_code)]
    pub fn ids() -> Vec<i32> {
        let asset_types = Self::list();
        asset_types.iter().map(|asset_type| asset_type.id).collect()
    }
}
