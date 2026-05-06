# MILESTONES.md

## Project Milestones (Ordered)
Follow TDD: write tests first, then implement. Run `cargo clippy && cargo fmt -- --check` after each milestone.

1. [x] **Project Initialization & Core Traits**
   - Initialize Cargo project (`cargo init --name modmanager`)
   - Define `Game` and `Mod` traits (per AGENTS.md)
   - *Completion check*: `cargo test` passes, `cargo clippy` clean

2. [ ] **Stardew Valley Game Module**
   - Implement `Game` trait for Stardew Valley (`games/stardew.rs`)
   - Add game path discovery (registry/system paths)
   - Write tests for game detection and path validation
   - *Completion check*: Stardew-specific tests pass

3. [ ] **Nexus API & Single Mod Support**
   - Implement Nexus API client (mock responses for tests)
   - Implement `Mod` trait with install/validate logic
   - Test single mod install to temp directory
   - *Completion check*: Mocked API tests pass, mod install works in tests

4. [ ] **Stock Game & Profile System**
   - Implement stock game folder management (isolate game dir from mods)
   - Implement profile create/switch/delete logic
   - Test profile isolation and stock folder sync
   - *Completion check*: Profile/stock tests pass

5. [ ] **Basic Iced UI**
   - Minimal Iced app using `iced::Sandbox` for state management
   - UI to list mods, toggle enable/disable, switch profiles
   - *Completion check*: `cargo run` launches working UI

6. [ ] **Collection Support**
   - Define native JSON collection format
   - Implement Nexus collection import
   - Test collection parsing and mod installation from collections
   - *Completion check*: Collection tests pass

7. [ ] **Multi-Game Extensibility**
   - Add a second game module (e.g., a simple test game)
   - Verify `Game` trait works across multiple games
   - Update game registry to discover all supported games
   - *Completion check*: Second game tests pass, registry works

8. [ ] **Polish & Hardening**
   - Full test suite run, fix all failures
   - Improve error handling and user-facing messages
   - Add config management (Nexus API key, paths)
   - *Completion check*: All tests pass, `cargo clippy` and `cargo fmt` clean
