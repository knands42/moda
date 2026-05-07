use crate::error::ModManagerError;

#[allow(dead_code)]
const NEXUS_API_BASE: &str = "https://api.nexusmods.com/v1";

#[allow(dead_code)]
pub struct NexusClient {
    api_key: String,
    client: reqwest::blocking::Client,
}

pub struct NexusModInfo {
    pub mod_id: u64,
    pub name: String,
    pub summary: String,
    pub game_domain: String,
}

#[allow(dead_code)]
pub struct NexusFileInfo {
    pub file_id: u64,
    pub name: String,
    pub category: String,
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
        _game_domain: &str,
        _mod_id: u64,
    ) -> Result<NexusModInfo, ModManagerError> {
        Err(ModManagerError::NexusApiError(
            "Get mod info not yet implemented".into(),
        ))
    }

    pub fn get_mod_files(
        &self,
        _game_domain: &str,
        _mod_id: u64,
    ) -> Result<Vec<NexusFileInfo>, ModManagerError> {
        Err(ModManagerError::NexusApiError(
            "Get mod files not yet implemented".into(),
        ))
    }

    pub fn download_mod(
        &self,
        _game_domain: &str,
        _mod_id: u64,
        _file_id: u64,
    ) -> Result<Vec<u8>, ModManagerError> {
        Err(ModManagerError::NexusApiError(
            "Download not yet implemented".into(),
        ))
    }
}
