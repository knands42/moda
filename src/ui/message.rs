#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Mods,
    Downloads,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    Reconcile,
    SyncAll,
    StageMod(String),
    UnstageMod(String),
    EnableMod(String),
    DisableMod(String),
}
