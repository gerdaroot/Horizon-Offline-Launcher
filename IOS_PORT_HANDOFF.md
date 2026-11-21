# iOS port: handoff for the next agent

> Read this first. It is the single source of truth for the iOS port. Keep the **Status** and
> **Change log** sections current whenever you touch the port.

## 0. TL;DR

* **Project:** Horizon = Minecraft Java 26.3 re-implemented in Rust (wgpu), ~40 crates, plus a GPUI
  desktop launcher (`launcher/`). This repo is the *Offline Launcher fork* (no Microsoft account
  needed to play; see `CLAUDE.md` top section).
* **Goal:** run the **game client** (`crates/horizon-client`, binary `horizon-client`) on iOS.
* **The launcher is NOT ported.** GPUI has no iOS backend, and an iOS app cannot download+exec
  code, spawn processes, or self-update. Its jobs are replaced on iOS (see section 3).
* **Verification status: NOTHING HAS BEEN COMPILED.** The session that started this port had no Rust
  toolchain, no Xcode and no network. Every change was made by reading code. **Your first job is
  `cargo check --target aarch64-apple-ios -p horizon-client` on a Mac** and fixing what it reports
  (section 6). Treat every edit as unverified.

## 1. What the project is (map)

See `CLAUDE.md` / `AGENTS.md` (identical) for the full crate map. What matters for iOS:

| Piece | Where | iOS relevance |
|---|---|---|
| Game binary | `crates/horizon-client/src/main.rs` -> `entry::main` -> `game::run_with` | **The thing we port.** `#![forbid(unsafe_code)]` in main.rs. |
| Renderer | `crates/horizon-client-render/src/gfx.rs` (wgpu 30, WGSL) | Metal backend; must be forced on iOS. |
| Window/event loop | `crates/horizon-client/src/game.rs` (`EventLoop::new()?.run_app(..)`, ~line 20984) | winit 0.30: iOS supported; `run_app` calls `UIApplicationMain`. |
| Input | `game.rs` `window_event`; `crates/horizon-client-input` | **No `WindowEvent::Touch` handling exists.** |
| Controller layer | `crates/horizon-client/src/game/pad.rs`, `pad_menu.rs`, `horizon-client-input/src/gamepad.rs` | Reusable: `PadState` (sticks, triggers, buttons). Touch controls feed this. |
| Resources | `Options::parse` (game.rs ~line 720) default `.local/horizon-client/26.3` | Prepared from the player's own Minecraft jar by `horizon-prepare`. |
| Prepare tool | `crates/horizon-prepare` | **Needs a JDK (javac + java).** Cannot run on iOS. |
| Launcher | `launcher/` (separate workspace) | Not ported. |

Upstream ROADMAP.md ("Android, iOS and the browser") lists the missing pieces for mobile:
**touch controls, a mobile interface, packaging, loading the player's own assets, preparing without
Java (#1750)**. Upstream already gated `horizon-dualsense` for iOS (#1813).

## 2. Hard constraints on iOS (why the design looks like it does)

1. **No JVM, no `fork/exec`.** `horizon-prepare` compiles Java oracles and runs vanilla exporters.
   => The prepared resource folder (`horizon-client/26.3/`, includes ~380 MB of sounds) must be built
   on a desktop with the launcher/`horizon-prepare` and **imported** onto the device (Files app /
   Finder file sharing). A pure-Rust prepare is the long-term fix (upstream #1750).
2. **Never ship Mojang files.** The `.ipa` must contain no jar/assets/prepared resources
   (`tools/check_public_tree.py` enforces this for the repo). The app reads them from its own
   `Documents/` sandbox.
3. **Sandbox:** the cwd is not writable. The game keeps saves/options/caches in the *working
   directory* (`--game-dir` default) => on iOS we `chdir` into `Documents/Horizon` at startup.
4. **No pointer capture, no keyboard, no mouse.** `set_cursor_grab` returns `NotSupported`.
5. **No Vulkan/GL.** Only Metal. wgpu's `gles`/`vulkan` features still *compile* (vulkan backend is
   compiled on all native targets), but must not be tried at runtime.
6. **Memory (jetsam):** iPhones kill apps at roughly 1.5-3 GB. Render distance, worker threads and
   LOD must default low on iOS.
7. **Lifecycle:** the surface is lost on background; the app is suspended without notice.
   Save on `Suspended`.
8. **Entitlements:** Increased Memory Limit is worth requesting; JIT is *not* needed (AOT Rust,
   but see section 7 on wasm mods).

## 3. Design

* Build the **existing bin** for `aarch64-apple-ios` (device) and `aarch64-apple-ios-sim`.
  No FFI entry point: winit's `run_app` starts `UIApplicationMain` from `main()`.
  Package the executable into `Horizon.app` (`ios/` folder, built by `tools/ios/build.sh`).
* New module `crates/horizon-client/src/ios.rs` (`cfg(target_os = "ios")`): sandbox setup
  (`chdir`, default `--resources`, home folder), mobile defaults.
* **Resources:** the app expects `Documents/horizon-client/26.3/` (same layout as `.local/horizon-client/26.3`).
  `UIFileSharingEnabled` + `LSSupportsOpeningDocumentsInPlace` make `Documents/` visible in the Files
  app. If missing, the existing `no_resources` screen is shown (needs an iOS-appropriate message).
* **Touch controls:** a virtual gamepad. `touch.rs` turns fingers into a `PadState` (left stick =
  move, right-side drag = look, buttons = jump/attack/use/sneak/inventory/pause/hotbar) that
  `poll_pads` merges next to `Pads::injected`. Menus work through the existing `pad_menu` cursor
  (and, later, direct tap -> synthetic mouse click).
* **Backends:** `backend_order()` returns `[Backends::METAL]` on iOS.

## 4. Status

(Updated at the end of the session that created this file; see Change log.)

## 5. Change log

(see bottom of file)

## 6. First steps on a Mac (do these before anything else)

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
cargo check --target aarch64-apple-ios -p horizon-client --no-default-features --features audio
```

Fix compile errors in this order of likelihood: (1) `cfg` branches missing an iOS arm
(`target_os = "macos"` vs `"linux"`/`windows` enumerations), (2) crates that don't build for iOS
(see `Cargo.toml` edits), (3) the new `ios.rs` / `touch.rs` modules (written blind).
Then `tools/ios/build.sh sim` and run in the iOS Simulator.

## 7. Known risks / open questions

* **WASM mods** (`horizon-mods`, wasmtime?) need JIT; iOS forbids it. Check the manifest: the game
  depends on `horizon-mods` with `default-features = false, features = ["index"]` so the runtime
  (`mods` feature) is not pulled in. Keep it that way on iOS.
* `third_party/wgpu-hal` (vendored 30.0.1) Metal backend: patched for HDR CoreGraphics names on
  macOS. Verify it compiles for iOS (it uses `target_vendor = "apple"` for deps, but CoreGraphics
  colour space names differ on iOS). HDR is off by default; disabling the HDR path on iOS is fine.
* mimalloc on iOS (`horizon-threads`): `target_vendor = "apple"` paths exist; if the build or the
  run is unstable, build with `--features system-allocator`.
* wgpu limits on Apple GPUs (A-series): the renderer takes `adapter.limits()` and masks optional
  features by `adapter.features()`, so it should adapt; `check_layers` (texture array layers) and
  the GPU culling / OIT features are untested on mobile GPUs.
* `std::process::Command` call sites (open folder, `ps`, crash system info) just fail on iOS;
  verify that they are `if let Ok`, not `unwrap`.
* App Store: apps that run user-supplied game data are fine; apps that download executable code
  are not. Distribute via TestFlight/AltStore/sideload first.
