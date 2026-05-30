# AGENTS.md

Project information on [README.md](./README.md).

## Testing Strategy
- Write tests **before** implementation (TDD approach)
- Test files mirror module structure in `tests/`
- Mock Nexus API responses for mod tests
- Use `tempfile` crate for profile/stock folder tests

## Important Constraints
- **Rust learning project**: Focus on architecture guidance + test skeletons, not full implementations
- **egui UI**: State management via `eframe::App` with modular pages under `src/ui/`
- **Nexus API**: Requires API key; store in `~/.config/moda/config.toml`
- **Collections**: Two formats planned — native JSON format + Nexus collections import

## Workflow
1. Define trait/tests for a module
2. Get tests passing (agent provides implementation hints only)
3. Integrate with existing modules
4. Run `cargo clippy && cargo fmt -- --check` before considering task complete
