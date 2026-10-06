# Chorus plugin (Rust, truce)

A chorus audio effect (modulated delay lines) built in Rust with the **truce** framework.
Primary target: **AU for Logic Pro**. Secondary: **VST3** (and CLAP, which truce scaffolds by default).

Status: backend (DSP + params) first. GUI is deliberately deferred.

<!-- Maintainer note: items marked (verify) came from docs I read in a chat, not from running code. Confirm them. -->

## Decisions already made

- Framework: **truce** (https://truce.audio, source: https://github.com/truce-audio/truce). One crate builds CLAP, VST3, AU v2, AU v3, standalone.
- Why truce: native AU v2/v3 support, `cargo truce validate` runs auval + pluginval + clap-validator, presets ship to Logic's AU factory list, GUI backends include Slint/egui.
- Rejected: nih-plug (VST3/CLAP only, in maintenance mode; AU would need the clap-wrapper-rs C++ bridge), beamer (pre-1.0, least proven), aura (explicitly ships no AU).
- Fallback if truce fails in Logic: nih-plug community fork + `clap-wrapper` crate (exports VST3 + AUv2 from a CLAP plugin).
- Logic Pro only hosts AU. Test AU first; VST3 is for other DAWs.
- Rust 1.92+ (edition 2024). AU builds require macOS.

## Commands

- `cargo install cargo-truce` (one time), `cargo truce doctor` (environment check)
- `cargo truce new <name>` scaffold (defaults: CLAP + VST3 + standalone)
- `cargo truce run` run standalone, no DAW needed
- `cargo truce install --au2` / `--au3` / `--vst3` / `--clap`
- `cargo truce validate` runs auval, pluginval, clap-validator on installed plugins
- `cargo truce reset-au` flush AU caches and restart pkd if a host shows stale or broken plugins
- `cargo truce log-stream-au` stream AU v3 logs (v3 runs out-of-process; NSLog does not reach the DAW log)
- All `cargo truce` commands build **release** by default; pass `--debug` for fast iteration.
- Tests: `cargo test`, `cargo clippy`. Audio regression tests use `truce_test::PluginDriver` (in-process, no DAW).
- Hot reload (`cargo truce build --shell`) is experimental; dev loop only.

## AU specifics (Logic)

- Enable AU in `Cargo.toml`: `[features] default = ["clap", "vst3", "au"]` (or pass `--au2`/`--au3` per install).
- Identifiers live in `truce.toml`: `[vendor] au_manufacturer` (4 chars), `[[plugin]] fourcc`, `au3_subtype`, `au_tag = "Effects"`.
- Keep `au3_subtype` **different** from `fourcc` if v2 and v3 are both installed. Logic is strict about component ID collisions.
- Validate manually: `auval -v aufx <fourcc> <manufacturer>` (type `aufx` = effect).

| | AU v2 | AU v3 |
|---|---|---|
| Toolchain | Xcode CLI tools | Full Xcode (`xcodebuild`) |
| Signing | Ad-hoc works locally | Developer ID Application required (ad-hoc rejected) |
| Install | `~/Library/Audio/Plug-Ins/Components/` (no sudo) | `/Applications/<Name>.app` (sudo, system-wide) |
| Host-resizable editor | **No** (fixed size) | Yes |
| Process model | In-process | Out-of-process (harder to debug) |

Default plan: develop on **AU v2** (simplest, no Developer ID needed). Revisit v3 only if a resizable GUI or iOS is wanted.

## Architecture

- `PluginLogic` trait covers both DSP and (later) the GUI via `editor()`. `#[derive(Params)]` + `#[param(...)]` declares params; `truce::plugin!` generates all format exports.
- Chorus needs per-channel delay-line state, so use the stateful `PluginLogic` (with `DspState`), not `PurePluginLogic`. (verify exact trait shape in https://rustdoc.truce.audio)
- Keep DSP in its own module with no truce types in its public API (pure `struct Chorus { ... }` with `process(&mut self, l, r)`), so it is unit-testable without the framework.
- truce API moves fast. Before assuming a signature, check https://truce.audio/docs/migrations/ and the changelog, and keep `Cargo.lock` committed.

## Real-time rules (audio thread)

- No allocation, locks, I/O, or logging inside `process()`.
- Allocate delay buffers in init/reset, sized from the sample rate. Re-size on sample-rate change.
- Smooth every parameter that touches the delay path (truce supports `smooth = "exp(ms)"` on params) to avoid zipper noise and clicks.
- Guard against NaN/denormals; clear all state in `reset()`.

## Initial DSP design (editable)

Stereo chorus: per channel, a short delay line read at a modulated fractional position.

- Delay time = base delay + LFO * depth. Typical ranges: base ~10-25 ms, depth ~1-5 ms.
- LFO sine or triangle, rate ~0.1-5 Hz; offset L/R LFO phase for stereo width.
- Fractional read with linear interpolation first, upgrade to cubic/Lagrange if artifacts are audible.
- Optional later: multiple voices (2-4) with detuned phases, small feedback, high-pass on the wet path.
- Dry/wet mix param; mix = 0 must be a bit-exact passthrough. Zero reported latency.

Initial params: `rate`, `depth`, `delay` (base), `mix`. Add `width` and `feedback` after the basics pass tests.

Reference implementation to read first: the chorus in https://github.com/truce-audio/reiss-mcpherson-effects (docs: https://truce.audio/docs/examples/reiss-chorus/).

## Testing plan

- Unit tests on the DSP struct: silence in gives silence out; mix = 0 equals input exactly; no NaN/inf with extreme params; output stays bounded.
- `truce_test::PluginDriver` tests: impulse response shape, 44.1/48/96 kHz, block sizes 1/64/512/4096, mono and stereo bus layouts, state save/restore.
- `cargo truce validate` must pass (auval is what Logic enforces) before any manual DAW test.
- Final check by hand: load the AU in Logic, automate a param, save/reload a project.

## GUI (deferred, do not start yet)

- For now use truce's built-in `GridLayout` editor (knobs for the params) or none.
- Later candidates: **Slint** (design in `.slint` markup) or **egui** (draw in code). Both go through truce's `editor()` and use `PluginContext` (`get_param`, `begin_edit` / `set_param` / `end_edit`, `automate`, `get_meter`).
- A web (HTML/React) UI is not built into truce (it is on their roadmap). It would need the raw-window-handle path plus wry. Treat it as high risk, especially inside AU hosts.
- Constraint: AU v2 editors cannot be resized by the host; design a fixed size unless using AU v3.

## Open questions (ask the user, do not guess)

- Plugin name, vendor name, and the 4-char `au_manufacturer` / `fourcc` codes.
- Is a Developer ID Application certificate available (needed for AU v3 and for distribution)?
- Which hosts besides Logic matter for VST3 (Ableton, Reaper, other)?
- Is this for personal use or distribution (affects signing, notarization, `cargo truce package`)?

## Working agreements for Claude Code

- Check the truce docs or rustdoc before writing framework code from memory.
- Run `cargo clippy` and `cargo test` after DSP changes; run `cargo truce validate` after changing identifiers or formats.
- Do not touch GUI code until the backend tests pass and the user says to start.
- Ask before changing the framework choice, AU identifiers, or signing setup.
