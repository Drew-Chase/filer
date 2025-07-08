use crate::internal_configuration::ic_db;
use anyhow::Result;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InternalConfiguration {
    pub has_done_first_run_setup: bool,
}

impl InternalConfiguration {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get() -> Self {
        let map = ic_db::get_all().await.unwrap_or_default();
        Self::from_hash_map(map)
    }

    pub async fn set_has_done_first_run_setup(&mut self, value: bool) -> Result<()> {
        self.has_done_first_run_setup = value;
        ic_db::set("has_done_first_run_setup", &value.to_string()).await?;
        Ok(())
    }

    fn to_hash_map(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        map.insert("has_done_first_run_setup".to_string(), self.has_done_first_run_setup.to_string());
        map
    }
    fn from_hash_map(map: std::collections::HashMap<String, String>) -> Self {
        let mut config = Self::new();
        if let Some(value) = map.get("has_done_first_run_setup") {
            config.has_done_first_run_setup = value.parse().unwrap_or(false);
        }
        config
    }
}
