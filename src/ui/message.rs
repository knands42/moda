#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    GameSelect,
    Mods,
    Downloads,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(Tab),
    SelectGame(String),
    Reconcile,
    SyncAll,
    StageMod(String),
    UnstageMod(String),
    EnableMod(String),
    DisableMod(String),
}
