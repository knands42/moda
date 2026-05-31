use std::sync::Arc;

use crate::mods::repository::ModRepository;
use crate::mods::types::{ModEntry, ModStatus, ReconciledMod};
use std::collections::HashMap;

pub struct ModState {
    mods: HashMap<String, ReconciledMod>,
    repository: Arc<dyn ModRepository>,
}

impl ModState {
    pub fn new(mods: HashMap<String, ReconciledMod>, repository: Arc<dyn ModRepository>) -> Self {
        Self { mods, repository }
    }
}

impl ModState {
    pub fn set_staged(&mut self, mod_entry: &ModEntry) {
        if let Some(m) = self.mods.get_mut(&mod_entry.name) {
            m.status = ModStatus::Staged;
            m.staging_entry = Some(mod_entry.clone());
            m.game_entry = None;
            let _ = self.repository.upsert_mod(&m.register_id, m);
        }
    }

    pub fn set_downloaded(&mut self, mod_entry: &ModEntry) {
        if let Some(m) = self.mods.get_mut(&mod_entry.name) {
            m.status = ModStatus::Downloaded;
            m.source_entry = Some(mod_entry.clone());
            m.staging_entry = None;
            m.game_entry = None;
            let _ = self.repository.upsert_mod(&m.register_id, m);
        }
    }

    pub fn set_enabled(&mut self, mod_entry: &ModEntry) {
        if let Some(m) = self.mods.get_mut(&mod_entry.name) {
            m.status = ModStatus::Enabled;
            m.game_entry = Some(mod_entry.clone());
            let _ = self.repository.upsert_mod(&m.register_id, m);
        }
    }

    pub fn set_unstaged(&mut self, name: &str) {
        if let Some(m) = self.mods.get_mut(name) {
            m.staging_entry = None;
            m.game_entry = None;
            m.status = ModStatus::Downloaded;
            let _ = self.repository.upsert_mod(&m.register_id, m);
        }
    }

    pub fn set_disabled(&mut self, name: &str) {
        if let Some(m) = self.mods.get_mut(name) {
            m.game_entry = None;
            m.status = ModStatus::Staged;
            let _ = self.repository.upsert_mod(&m.register_id, m);
        }
    }

    pub fn remove(&mut self, name: &str) {
        let register_id = self
            .mods
            .get(name)
            .map(|m| m.register_id.clone())
            .unwrap_or_default();
        self.mods.remove(name);
        let _ = self.repository.remove_mod(&register_id, name);
    }

    pub fn get_mods(&self) -> impl Iterator<Item = &ReconciledMod> {
        self.mods.values()
    }

    // TODO: Precalculate snapshot based on events
    pub fn snapshot(&self) -> Vec<ReconciledMod> {
        let mut mods: Vec<_> = self.mods.values().cloned().collect();
        mods.sort_by(|a, b| a.name.cmp(&b.name));
        mods
    }

    pub fn get_mod(&self, name: &str) -> Option<&ReconciledMod> {
        self.mods.get(name)
    }
}
