# Installers Refactor Summary

This document summarizes the changes made to factor out installation logic and keep legacy imports working.

## What Changed

- Added two new crates under `installers/`:
  - `installers/agents` for agent and IDE hook installers.
  - `installers/git_clients` for git client preference installers.
- Copied existing installer logic into those new crates and wired them into the workspace.
- Added thin shim modules under `crates/git-ai/src/mdm/` that re-export from the new crates.
- Kept legacy module paths available so existing imports do not need to change.
- Updated `installers/README.md` to document the structure.
- Fixed the `jsonc-parser` dependency name in the new crates.
- Adjusted OpenCode embed path to keep using the shared `agent-support` source.
- Simplified diff generation inside the installers agents utils to avoid non-local dependencies.

## Why These Changes Were Made

- The new crates isolate installer logic and make it easier to maintain or reuse.
- Shim modules preserve compatibility with existing code and downstream consumers.
- The README addition documents the new layout for future contributors.
- Dependency fixes were required to compile the new crates.
- The OpenCode embed path keeps existing plugin content centralized.
- Simplifying diff generation removes a dependency on unrelated internal modules.

## Key Files and Directories

- `installers/agents/` and `installers/git_clients/`
- `crates/git-ai/src/mdm/` (shim modules)
- `installers/README.md`
