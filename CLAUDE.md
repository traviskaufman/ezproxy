# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

EZProxy is a keyboard shortcuts tool for browser address bars. It runs a local HTTP server that intercepts queries and redirects to configured URLs. Users define shortcuts in a config file (e.g., `m` → Gmail, `npm [query]` → NPM search) and set the server as their browser's custom search engine.

## Repository Structure

This is a multi-project repo:

- **`ezproxy-core/`** — Main Rust backend: HTTP server, config parsing, rule evaluation.
- **`ezproxy-osx/`** — Native macOS menu bar app (SwiftUI) that bundles and spawns the `ezproxy` binary. The intended distribution method.

## Build & Test Commands

### Rust Backend (run from `ezproxy-core/`)

```bash
cargo check                          # Type check
cargo test                           # Run all tests (unit + integration)
cargo test <test_name>               # Run a single test
cargo fmt --all -- --check           # Format check
cargo clippy -- -D warnings          # Lint
cargo build --release                # Release build
cargo run -- <config-file> [--port PORT]  # Run server (default port 5050)
```

### Acceptance Tests (run from `acceptance-tests/`)

```bash
npm test                                              # Run all acceptance tests
npm run test:native-installer                         # Run native-installer tests only
```

## Architecture

### Core Request Pipeline

```
Browser → GET /?q=<input> → CommandParser → Command {name, args}
                                                ↓
                                        Redirector.evaluate()
                                                ↓
                                    Lookup rule by command name
                                                ↓
                                    Rule.produce_uri() → Uri
                                                ↓
                                    HTTP 302 Redirect
```

### Key Source Files

| File | Responsibility |
|------|---------------|
| `ezproxy-core/src/main.rs` | Hyper/Tokio HTTP server, `CommandParser`, `Redirector`, request handling |
| `ezproxy-core/src/config.rs` | Parses config files (`<keyword> = <url>` format), implements `ConfigRule` with token substitution (`{ARGS}`, `{ALL}`) |
| `ezproxy-core/src/rules.rs` | `Rule` trait definition, `DEFAULT_RULE_KEY` (`_`) for fallback |
| `ezproxy-core/src/lib.rs` | Crate root, re-exports `config` and `rules` modules |
| `ezproxy-core/tests/integration_test.rs` | E2E tests that spawn the server process and validate redirects |

### Key Concepts

- **Rule trait**: Extensible interface for producing redirect URIs. Config-based rules implement this; code-based rules can be added.
- **Token substitution**: `{ARGS}` = everything after the command, `{ALL}` = command + args combined.
- **Fallback rule**: The `_` key is used when no shortcut matches the input.
- **Config format**: Simple text file, one rule per line: `keyword = https://example.com/{ARGS}`

### Dependencies

- **hyper** 0.14 + **tokio** for async HTTP
- **clap** 3.2 for CLI args
- **regex** + **lazy_static** for config parsing
- Acceptance tests: **Vitest**, **TypeScript** 5.8
