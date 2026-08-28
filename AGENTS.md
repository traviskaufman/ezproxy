# AGENTS.md

## Project: EZProxy

Keyboard shortcuts for your browser address bar. A local HTTP server evaluates shortcuts and returns 302 redirects.

## Architecture

Single git repo (`traviskaufman/ezproxy`); `.github/`, `LICENSE`, `.gitignore` live at the root. CI runs cargo with `working-directory: ezproxy-core`.

- `ezproxy-core/` — Rust HTTP server (hyper + tokio). Config parsing, rule evaluation, redirect response.
- `ezproxy-osx/` — SwiftUI menu bar app. `ProxyProcess` spawns the bundled `ezproxy` binary as a child process (`~/ezproxy.txt --port 5050`); `EZProxyOSXApp` is a `MenuBarExtra` with status + Start/Stop/Restart/Quit.
  - `LSUIElement = YES` (no dock icon), app sandbox disabled so the child can bind `:5050` and read `~/ezproxy.txt`.
  - Xcode "Bundle ezproxy binary" run-script phase runs `cargo build --release` in `ezproxy-core/` and copies the binary into `Contents/MacOS/`, then codesigns it with `EXPANDED_CODE_SIGN_IDENTITY` (ad-hoc `-` when signing is disabled) so the app's own signature verifies.
  - Icons live in `Assets.xcassets`: `AppIcon` (Finder) and `MenuBarIcon` (18pt template glyph). Sources were rendered with `rsvg-convert` from SVG.

## Key Concepts

- **Config format**: `<keyword> = <url>` with `{ARGS}` and `{ALL}` token substitution.
- **Fallback rule**: `_` key matches when no shortcut matches.
- **Request flow**: `GET /?q=<input>` → parse command → lookup rule → 302 redirect.
- **Default port**: 5050.

## Build

- Rust: `cargo check`, `cargo test`, `cargo clippy -- -D warnings` (from `ezproxy-core/`)
- macOS app: `cd ezproxy-osx/EZProxyOSX && xcodebuild -scheme EZProxyOSX -configuration Debug -allowProvisioningUpdates build` (signs with the "Apple Development" cert, team 2NM78QK3X5). Requires `cargo` in `~/.cargo/bin`.

## Gotchas

- `ProxyProcess` starts the child in `init` and stops it on `NSApplication.willTerminateNotification`, so any quit path (menu, AppleScript, logout) kills the child.
- If something else holds `:5050` the child exits immediately and the menu shows "Stopped"; there is deliberately no auto-restart (it would crash-loop).
- The launchd agent `com.github.traviskaufman.ezproxy` (from the README) is unloaded on this machine in favor of the app.
