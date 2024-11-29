use crate::model::action_type::ActionTypeModel;
use axum_kit::{postgres, AppResult};
use std::sync::OnceLock;

static ACTION_TYPE: OnceLock<Vec<ActionTypeModel>> = OnceLock::new();

pub struct ActionTypeService;

impl ActionTypeService {
    pub async fn init() -> AppResult<()> {
        let action_types = ActionTypeModel::fetch_all(postgres::conn()).await?;
        let _ = ACTION_TYPE
            .set(action_types)
            .map_err(|_| "Failed to initialize ACTION_TYPE");
        Ok(())
    }

    pub fn list() -> &'static Vec<ActionTypeModel> {
        ACTION_TYPE.get().expect("ACTION_TYPE is not initialized")
    }

    #[allow(dead_code)]
    pub fn is_active(id: i32) -> bool {
        let action_types = Self::list();
        action_types.iter().any(|action_type| action_type.id == id)
    }

    #[allow(dead_code)]
    pub fn by_id(id: i32) -> Option<&'static ActionTypeModel> {
        let action_types = Self::list();
        action_types
            .iter()
            .find(|&action_type| action_type.id == id)
    }
}
