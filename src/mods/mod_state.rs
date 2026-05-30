use crate::mods::catalog::{ModStatus, ReconciledMod};
use crate::mods::types::ModEntry;
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
        let base = mod_entry
            .name
            .strip_suffix(".zip")
            .unwrap_or(&mod_entry.name);
        if let Some(m) = self.mods.get_mut(base) {
            m.status = ModStatus::Staged;
            m.staging_entry = Some(mod_entry.clone());
            m.game_entry = None;
        }
    }

    pub fn set_downloaded(&mut self, mod_entry: &ModEntry) {
        let base = mod_entry
            .name
            .strip_suffix(".zip")
            .unwrap_or(&mod_entry.name);
        if let Some(m) = self.mods.get_mut(base) {
            m.status = ModStatus::Downloaded;
            m.source_entry = Some(mod_entry.clone());
            m.staging_entry = None;
            m.game_entry = None;
        }
    }

    pub fn set_enabled(&mut self, mod_entry: &ModEntry) {
        let base = mod_entry
            .name
            .strip_suffix(".zip")
            .unwrap_or(&mod_entry.name);
        if let Some(m) = self.mods.get_mut(base) {
            m.status = ModStatus::Enabled;
            m.game_entry = Some(mod_entry.clone());
        }
    }

    pub fn set_unstaged(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        if let Some(m) = self.mods.get_mut(base) {
            m.staging_entry = None;
            m.game_entry = None;
            m.status = ModStatus::Downloaded;
        }
    }

    pub fn set_disabled(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        if let Some(m) = self.mods.get_mut(base) {
            m.game_entry = None;
            m.status = ModStatus::Staged;
        }
    }

    pub fn remove(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        self.mods.remove(base);
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
        let base = name.strip_suffix(".zip").unwrap_or(name);
        self.mods.get(base)
    }
}
