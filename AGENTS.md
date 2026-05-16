# AGENTS.md

## Project Overview
Mod manager for Linux built with Rust + Iced. Starting with Stardew Valley, designed for multi-game extensibility.

## Developer Commands
```bash
cargo init --name moda                # Initialize project (if not done)
cargo run --bin moda                  # Run the app
cargo test                            # Run all tests
cargo test <test_name>                # Run single test
cargo clippy -- -D warnings          # Lint (fail on warnings)
cargo fmt -- --check                 # Check formatting
```

## Architecture

### Core Design Principles
- **Game-agnostic core**: Game-specific logic lives in separate modules/crates, not in core
- **Stock game approach**: Keep game folder clean; manage mods in a separate library folder (like Wabbajack)
- **Profile-based**: Support multiple profiles per game from the start
- **Mod collections**: Support both native JSON format and Nexus collections import

### Current Crate Structure
```
moda/
├── Cargo.toml
├── src/
│   ├── main.rs              # Iced app entrypoint
│   ├── lib.rs               # Re-exports public modules
│   ├── config.rs            # Config loading from ~/.config/modmanager/config.toml
│   ├── error.rs             # Error handling
│   ├── games/
│   │   ├── mod.rs           # Game trait definition
│   │   └── stardew.rs       # Stardew Valley implementation
│   ├── mods/
│   │   ├── mod.rs           # Re-exports Installer & NexusClient
│   │   ├── nexus.rs         # Nexus API client
│   │   └── installer.rs     # Mod installation logic (ModSource enum + Installer struct)
│   └── profiles/
│       └── mod.rs           # Profile management (stub)
└── tests/
    ├── mod.rs
    ├── games/
    │   ├── mod.rs
    │   └── stardew_test.rs
    └── mods/
        ├── mod.rs
        ├── nexus.rs
        └── installer_test.rs
```

## Testing Strategy
- Write tests **before** implementation (TDD approach)
- Test files mirror module structure in `tests/`
- Mock Nexus API responses for mod tests
- Use `tempfile` crate for profile/stock folder tests

## Important Constraints
- **Rust learning project**: Focus on architecture guidance + test skeletons, not full implementations
- **Iced UI**: State management via `iced::Application` or `Sandbox`
- **Nexus API**: Requires API key; store in `~/.config/moda/config.toml`
- **Collections**: Two formats planned — native JSON format + Nexus collections import

## Workflow
1. Define trait/tests for a module
2. Get tests passing (agent provides implementation hints only)
3. Integrate with existing modules
4. Run `cargo clippy && cargo fmt -- --check` before considering task complete
