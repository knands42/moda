use crate::mods::mod_registry::{ModStatus, ReconciledMod};

#[derive(Clone, Default)]
pub struct ModState {
    // TODO: Use a map instead of a vector
    pub mods: Vec<ReconciledMod>,
}

impl ModState {
    pub fn set_staged(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        if let Some(m) = self.mods.iter_mut().find(|m| m.name == base) {
            m.status = ModStatus::Staged;
        }
    }

    pub fn set_unstaged(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        if let Some(m) = self.mods.iter_mut().find(|m| m.name == base) {
            m.status = ModStatus::Downloaded;
        }
    }

    pub fn set_enabled(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        if let Some(m) = self.mods.iter_mut().find(|m| m.name == base) {
            m.status = ModStatus::Enabled;
        }
    }

    pub fn set_disabled(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        if let Some(m) = self.mods.iter_mut().find(|m| m.name == base) {
            m.status = ModStatus::Staged;
        }
    }

    pub fn remove(&mut self, name: &str) {
        let base = name.strip_suffix(".zip").unwrap_or(name);
        self.mods.retain(|m| m.name != base);
    }
}
