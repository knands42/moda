use crate::error::ModManagerError;
use serde::Deserialize;

#[allow(dead_code)]
fn nexus_api_base() -> String {
    std::env::var("NEXUS_API_BASE").unwrap_or_else(|_| "https://api.nexusmods.com/v3".to_string())
}

pub struct NexusClient {
    api_key: String,
    client: reqwest::blocking::Client,
}

#[derive(Deserialize)]
struct NexusApiResponse<T> {
    data: T,
}

#[derive(Deserialize)]
pub struct NexusModInfo {
    pub id: String,
    pub game_scoped_id: String,
    pub game_id: String,
    pub name: Option<String>,
}

impl NexusClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn search_mods(
        &self,
        _game_domain: &str,
        _query: &str,
    ) -> Result<Vec<NexusModInfo>, ModManagerError> {
        Err(ModManagerError::NexusApiError(
            "Search not yet implemented".into(),
        ))
    }

    pub fn get_mod_info(
        &self,
        game_domain: &str,
        mod_id: u64,
    ) -> Result<NexusModInfo, ModManagerError> {
        let url = format!("{}/games/{game_domain}/mods/{mod_id}", nexus_api_base());

        let response = self
            .client
            .get(&url)
            .header("apikey", &self.api_key)
            .send()
            .map_err(|e| ModManagerError::NexusApiError(e.to_string()))?;

        let wrapper: NexusApiResponse<NexusModInfo> = response
            .json()
            .map_err(|e| ModManagerError::NexusApiError(e.to_string()))?;

        Ok(wrapper.data)
    }

    pub fn download_mod(
        &self,
        game_domain: &str,
        mod_id: u64,
        file_id: u64,
    ) -> Result<Vec<u8>, ModManagerError> {
        let url = format!(
            "{}/games/{game_domain}/mods/{mod_id}files/{file_id}",
            nexus_api_base()
        );

        self.client
            .get(&url)
            .header("apikey", &self.api_key)
            .send()
            .map_err(|e| ModManagerError::NexusApiError(e.to_string()))?;

        todo!()
    }
}
