---
name: slot-development
description: Contributor workflow for cartridge-gg/slot. Use when implementing or reviewing Rust CLI/runtime changes, schema syncing, and lint/test checks in this repository.
---

# Slot Development

Use this skill to deliver changes in `cartridge-gg/slot` with correct lint/test sequencing and e2e awareness.

## Core Workflow

1. Build and run focused command checks while iterating:
   - `cargo build`
   - `cargo run -- <command>`
2. Run formatting and lint checks:
   - `./scripts/rust_fmt.sh`
   - `./scripts/clippy.sh`
3. Run integration and e2e checks when behavior changes:
   - `./scripts/e2e.sh`
   - Note: this script creates and deletes temporary deployments
4. Refresh external schema when API-affecting work is included:
   - `./scripts/pull_schema.sh`

## PR Checklist

- Mention if e2e was run and summarize outcome.
- Call out schema updates when GraphQL/API behavior changes.
- Keep validation commands and results in PR notes.
