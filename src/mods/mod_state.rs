use crate::mods::mod_registry::ReconciledMod;

pub struct ModState {
    pub mods: Vec<ReconciledMod>
}

impl ModState {
    pub fn set_staged(&mut self, name: &str) {}
    pub fn set_enabled(&mut self, name: &str) {}
    pub fn remove(&mut self, name: &str) {}
}