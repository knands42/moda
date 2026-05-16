use crate::mods::mod_registry::{ModStatus, ReconciledMod};
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct ModState {
    mods: HashMap<String, ReconciledMod>,
}

impl ModState {
    pub fn new(mods: HashMap<String, ReconciledMod>) -> Self {
        Self { mods }
    }

    pub fn from_vec(mods: Vec<ReconciledMod>) -> Self {
        Self {
            mods: mods.into_iter().map(|m| (m.name.clone(), m)).collect(),
        }
    }
}

impl ModState {
    pub fn set_staged(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        if let Some(m) = self.mods.get_mut(base) {
            m.status = ModStatus::Staged;
        }
    }

    pub fn set_downloaded(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        if let Some(m) = self.mods.get_mut(base) {
            m.status = ModStatus::Downloaded;
        }
    }

    pub fn set_enabled(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        if let Some(m) = self.mods.get_mut(base) {
            m.status = ModStatus::Enabled;
        }
    }

    pub fn remove(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        self.mods.remove(base);
    }

    pub fn get_mods(&self) -> impl Iterator<Item = &ReconciledMod> {
        self.mods.values()
    }

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
