# AGENTS.md

## Project Overview
Mod manager for Linux built with Rust + Iced. Starting with Stardew Valley, designed for multi-game extensibility.

## Developer Commands
```bash
cargo init --name modmanager          # Initialize project (if not done)
cargo run --bin modmanager            # Run the app
cargo tdd-scaffold                           # Run all tests
cargo tdd-scaffold --tdd-scaffold <test_name>        # Run single tdd-scaffold file
cargo clippy -- -D warnings          # Lint (fail on warnings)
cargo fmt -- --check                 # Check formatting
```

## Architecture

### Core Design Principles
- **Game-agnostic core**: Game-specific logic lives in separate modules/crates, not in core
- **Stock game approach**: Keep game folder clean; manage mods in a separate library folder (like Wabbajack)
- **Profile-based**: Support multiple profiles per game from the start
- **Mod collections**: Support both native JSON format and Nexus collections import

### Suggested Crate Structure
```
modmanager/
├── Cargo.toml
├── src/
│   ├── main.rs              # Iced app entrypoint
│   ├── lib.rs               # 
│   ├── app.rs               # Main application state/ui
│   ├── error.rs             # Error handling
│   ├── games/
│   │   ├── mod.rs           # Game trait definition
│   │   ├── stardew.rs       # Stardew Valley implementation
│   │   └── registry.rs      # Game registration/discovery
│   ├── mods/
│   │   ├── mod.rs           # Mod struct + trait
│   │   ├── nexus.rs         # Nexus API client
│   │   └── installer.rs     # Mod installation logic
│   ├── profiles/
│   │   └── mod.rs           # Profile management
│   └── stock/
│       └── mod.rs           # Stock game folder management
└── tests/
    ├── games/*.rs
    ├── mods/*.rs
    ├── profiles/*.rs
    └── stock/*.rs
```

### Key Traits to Define Early
```rust
// games/mod.rs
pub trait Game {
    fn name(&self) -> &str;
    fn game_path(&self) -> PathBuf;
    fn mods_path(&self) -> PathBuf;
    fn discover_path() -> Option<PathBuf>;
    fn registry_id() -> &'static str;
}

// mods/mod.rs
pub trait Mod {
    fn id(&self) -> &str;
    fn install(&self, target: &Path) -> Result<()>;
    fn validate(&self) -> Result<()>;
}
```

## Testing Strategy
- Write tests **before** implementation (TDD approach)
- Test files mirror module structure in `tests/`
- Mock Nexus API responses for mod tests
- Use `tempfile` crate for profile/stock folder tests

## Important Constraints
- **Rust learning project**: Focus on architecture guidance + test skeletons, not full implementations
- **Iced UI**: State management via `iced::Application` or `Sandbox`
- **Nexus API**: Requires API key; store in `~/.config/modmanager/config.toml`
- **Collections**: Two formats planned — native JSON format + Nexus collections import

## Workflow
1. Define trait/tests for a module
2. Get tests passing (agent provides implementation hints only)
3. Integrate with existing modules
4. Run `cargo clippy && cargo fmt -- --check` before considering task complete
