use crate::mods::types::{ModEntry, ModStatus, ReconciledMod};
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct ModState {
    mods: HashMap<String, ReconciledMod>,
}

impl ModState {
    pub fn new(mods: HashMap<String, ReconciledMod>) -> Self {
        Self { mods }
    }
}

impl ModState {
    pub fn set_staged(&mut self, mod_entry: &ModEntry) {
        if let Some(m) = self.mods.get_mut(&mod_entry.name) {
            m.status = ModStatus::Staged;
            m.staging_entry = Some(mod_entry.clone());
            m.game_entry = None;
        }
    }

    pub fn set_downloaded(&mut self, mod_entry: &ModEntry) {
        if let Some(m) = self.mods.get_mut(&mod_entry.name) {
            m.status = ModStatus::Downloaded;
            m.source_entry = Some(mod_entry.clone());
            m.staging_entry = None;
            m.game_entry = None;
        }
    }

    pub fn set_enabled(&mut self, mod_entry: &ModEntry) {
        if let Some(m) = self.mods.get_mut(&mod_entry.name) {
            m.status = ModStatus::Enabled;
            m.game_entry = Some(mod_entry.clone());
        }
    }

    pub fn set_unstaged(&mut self, name: &str) {
        if let Some(m) = self.mods.get_mut(name) {
            m.staging_entry = None;
            m.game_entry = None;
            m.status = ModStatus::Downloaded;
        }
    }

    pub fn set_disabled(&mut self, name: &str) {
        if let Some(m) = self.mods.get_mut(name) {
            m.game_entry = None;
            m.status = ModStatus::Staged;
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.mods.remove(name);
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
