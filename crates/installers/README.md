# Installers Structure

This repo factors installation logic into two crates under the top-level `installers/` directory.

## Layout

- `installers/agents/`
  - Contains agent/IDE hook installers and JetBrains plugin logic.
  - Primary entry points are in `installers/agents/src/lib.rs` and `installers/agents/src/mdm/`.
- `installers/git_clients/`
  - Contains git client preference installers (e.g., Fork, Sublime Merge).
  - Primary entry points are in `installers/git_clients/src/lib.rs` and `installers/git_clients/src/mdm/`.

## Compatibility Shims

Legacy paths under `crates/git-ai/src/mdm/` remain in place. They re-export items from the new crates so existing imports do not need to change.

Examples:
- `crates/git-ai/src/mdm/hook_installer.rs` re-exports from `installers/agents`.
- `crates/git-ai/src/mdm/git_client_installer.rs` re-exports from `installers/git_clients`.
- `crates/git-ai/src/mdm/utils.rs` re-exports shared installer utilities.

## Notes

- The OpenCode plugin source is still stored under `crates/git-ai/agent-support/` and is embedded by the agents crate.
- The shims are intentionally thin and should not grow new logic.
