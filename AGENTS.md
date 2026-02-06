# Repository Guidelines

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Slot is a CLI tool and execution layer for the Dojo ecosystem that manages rapid provisioning of low-latency, dedicated, provable execution contexts. It provides horizontal scalability to blockchain applications through managed sequencing, proving, and settlement.

## Tech Stack

- **Language**: Rust (workspace with two crates: `slot-cli` and `slot`)
- **Web Framework**: Axum for HTTP services
- **GraphQL**: Comprehensive GraphQL client integration with generated code
- **Blockchain**: Starknet SDK, Cartridge Controller Account SDK
- **External Services**: Katana (sequencer) and Torii (indexer) from Dojo ecosystem

## Development Commands

### Core Development Tasks
```bash
# Linting (strict - warnings treated as errors)
./scripts/clippy.sh

# Code formatting check
./scripts/rust_fmt.sh

# Code formatting fix
./scripts/rust_fmt_fix.sh

# End-to-end testing (creates/tests/deletes deployments)
./scripts/e2e.sh

# Update GraphQL schema from API
./scripts/pull_schema.sh
```

### Build Commands
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run CLI locally
cargo run -- <command>
```

### Testing
- E2E tests create temporary deployments, test functionality, and clean up
- Tests verify both Katana and Torii service deployments
- Health checks ensure services are responding before proceeding

## Architecture

### Command Structure
The CLI is organized into main command groups:
- `slot auth` - Authentication and session management
- `slot deployments` (alias: `d`) - Katana/Torii service management  
- `slot teams` - Team collaboration features
- `slot paymaster`/`slot paymasters` - Gas fee management

### Core Components
- **CLI Layer** (`cli/src/command/`) - Command implementations organized by domain
- **Core Library** (`slot/src/`) - Shared functionality including GraphQL client, account management, API client
- **GraphQL Integration** - Generated client code from comprehensive schema (37k+ lines)
- **Service Management** - Configuration-driven deployment of Katana/Torii services

### Key Files
- `slot/src/graphql/schema.graphql` - GraphQL schema (regenerate Rust code after changes)
- `cli/src/command/` - Command implementations
- `slot/src/api.rs` - Core API client with authentication

## Environment Variables
- `SLOT_DISABLE_AUTO_UPDATE` - Disable automatic updates
- `SLOT_FORCE_AUTO_UPDATE` - Force updates without confirmation  
- `CARTRIDGE_API_URL` - Override API endpoint
- `CARTRIDGE_KEYCHAIN_URL` - Override keychain endpoint

## Development Notes

### Service Deployments
- Katana: Starknet sequencer with configurable block times and account predeployment
- Torii: Indexer requiring world address configuration
- Both support tier scaling (basic to epic) for performance requirements

### Authentication
All API operations require authentication via `slot auth login` which handles Cartridge Controller integration.

## Installation & Distribution
- Users install via `curl -L https://slot.cartridge.sh | bash`
- Auto-update mechanism built into CLI
- Multi-stage Docker build for containerized deployment

## Agent Tooling

- **Pre-commit hooks:** run `bin/setup-githooks` (configures `core.hooksPath` for this repo).

- **Source of truth:** `.agents/`.
- **Symlinks:** `CLAUDE.md` is a symlink to this file (`AGENTS.md`). Editor/agent configs should symlink skills from `.agents/skills`.
- **Skills install/update:**

```bash
npm_config_cache=/tmp/npm-cache npx -y skills add https://github.com/cartridge-gg/agents   --skill create-pr create-a-plan   --agent claude-code cursor   -y
```

- **Configs:**
  - `.agents/skills/` (canonical)
  - `.claude/skills` -> `../.agents/skills`
  - `.cursor/skills` -> `../.agents/skills`

## Code Review Invariants

- No secrets in code or logs.
- Keep diffs small and focused; avoid drive-by refactors.
- Add/adjust tests for behavior changes; keep CI green.
- Prefer check-only commands in CI (`format:check`, `lint:check`) and keep local hooks aligned.
- For Starknet/Cairo/Rust/crypto code: treat input validation, authZ, serialization, and signature/origin checks as **blocking** review items.
