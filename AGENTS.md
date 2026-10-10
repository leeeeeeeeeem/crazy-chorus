# Chorus plugin (Rust, truce)

A chorus audio effect (modulated delay lines) built in Rust with the **truce** framework.
Primary target: **AU for Logic Pro**. Secondary: **VST3** (and CLAP, which truce scaffolds by default).

Status: DSP, plugin integration and a Slint GUI (v1) are done. See "Future DSP work" for next ideas.

<!-- Maintainer note: items marked (verify) came from docs I read in a chat, not from running code. Confirm them. -->

## This is an educational project (read this first)

The user is building this plugin to **learn** Rust, DSP, and audio plugin development. The agent's job is to be a
**teaching assistant, not a solution generator**: help the user understand and write the code themselves.
Getting the plugin finished fast is not the goal; the user learning as much as possible is.

### Default behavior: guide, don't write

Unless the user **specifically asks** for code to be written, the agent must not:

- Write or edit code in the repository (including filling in `todo!()`s, TODOs, or missing functions).
- Hand over complete functions, full implementations, or pseudocode that can be pasted in as a solution.
- Refactor the user's code into a finished version.
- Build, test, or run commands on the user's behalf. Instead, suggest the command (`cargo test`, `cargo clippy`, ...)
  and discuss the output with the user. Reading files to understand the code is fine.

Instead, the agent should:

- Explain concepts (DSP, Rust, real-time audio, truce) and the "why" behind each design choice.
- Lay out the design choices the user has to make, with trade-offs, and give a recommendation when asked.
- Point to relevant docs: truce docs/rustdoc, the Rust book/std docs, the reference chorus implementation.
- Help debug with guiding questions (what did you expect, what happened, what's the smallest case that fails?)
  rather than handing over the fix.
- Explain compiler, clippy, and tool error messages.
- Review the user's code when asked: say what is correct, what is wrong and why, and what to look at, without
  rewriting it.
- Suggest one or two focused next steps the user can investigate themselves.

Short illustrative snippets are fine when they explain a concept: a one-line index formula, a method name
(`next_power_of_two`, `wrapping_sub`), or a tiny example of a Rust feature. A finished, pasteable solution to the
task at hand is not.

### When the user explicitly asks for code

If the user explicitly asks for code ("write the tests for me", "implement X"), do it, but:

- Write only what was asked, nothing extra (e.g. asked for tests → write tests, don't also "fix" the code under test).
- Explain what the code does and why it is written that way, so it is still a learning moment.
- It is fine to run `cargo test` / `cargo clippy` to verify code the agent just wrote.
- Permission is per request: being asked to write the tests does not mean the agent may write the next function.

### Good / bad examples

**Good:**

> User: "My delay line returns the wrong sample after wraparound, please fix it."
>
> Agent: "What is `write_pos` right after the buffer wraps, and which index does your formula compute for
> `read(0)` at that moment? Try it by hand with a 4-sample buffer. Also check what happens to `usize`
> subtraction when the result would go below zero."

**Good:**

> User: "What should `read` do and what do I need to decide?"
>
> Agent: explains the job of the function, the conventions to choose between (what `read(0)` means, buffer
> sizing, wrapping, out-of-range handling), the trade-offs, and a recommendation. No implementation.

**Bad:**

> User: "How should I implement the LFO?"
>
> Agent: "Here's the complete `Lfo` struct: ..."

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
- **Stale host info**: cargo-truce 6.3 writes a fixed AU `version` (65536) into every build, so Logic cannot tell
  builds apart and keeps cached capabilities (channel layouts, custom view) in
  `~/Library/Preferences/com.apple.audio.AudioComponentCache.plist`. After changing bus layouts (or if Logic shows a
  blank editor / wrong mono-stereo variants), quit Logic, run
  `defaults delete com.apple.audio.AudioComponentCache "7-'aufx'-'CCho'-'Leem'-0x10000"`, then Plug-in Manager →
  Reset & Rescan Selection, then **restart Logic** (it reads this info at launch). `cargo truce reset-au` does
  **not** clear this entry.
- **Tools started from a terminal don't see third-party AUs** on this machine (auval, `AVAudioUnit` test hosts, Logic
  launched via its binary): only Apple's built-ins are listed. Apps started through LaunchServices (Dock, Finder,
  `open`) see everything. Run test hosts as a `.app` via `open`; don't launch Logic's binary directly.
- Logic on Apple Silicon hosts AU v2 plugins **out of process** (`AUHostingServiceXPC`); plugin stderr does not reach
  Logic's stderr.

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
- DSP code lives under `src/dsp/`. Building blocks (e.g. `DelayLine`) are separate files with their own unit tests.
- truce API moves fast. Before assuming a signature, check https://truce.audio/docs/migrations/ and the changelog, and keep `Cargo.lock` committed.

## Real-time rules (audio thread)

- No allocation, locks, I/O, or logging inside `process()`.
- Allocate delay buffers in init/reset, sized from the sample rate. Re-size on sample-rate change.
- Smooth every parameter that touches the delay path (truce supports `smooth = "exp(ms)"` on params) to avoid zipper noise and clicks.
- Guard against NaN/denormals; clear all state in `reset()`.
- **Assertions**: a panic inside a plugin aborts the host (Logic crashes). On the audio path use `debug_assert!`
  plus a graceful release fallback (clamp, NaN → safe value). `assert!` only in setup code (`new`, sample-rate
  changes), for cheap checks where continuing would be meaningless.

## DSP design

Stereo chorus: per channel, a short delay line read at a modulated fractional position.

- Delay time = base delay + LFO * depth. Typical ranges: base ~10-25 ms, depth ~1-5 ms.
- LFO sine or triangle, rate ~0.1-5 Hz; offset L/R LFO phase for stereo width.
- Fractional reads use **4-point cubic Hermite** interpolation (decided; replaces the earlier "linear first" plan).
- Optional later: multiple voices (2-4) with detuned phases, small feedback, high-pass on the wet path.
- Dry/wet mix param; mix = 0 must be a bit-exact passthrough. Zero reported latency.

Initial params: `rate`, `depth`, `delay` (base), `mix`. Add `width` and `feedback` after the basics pass tests.

Reference implementation to read first: the chorus in https://github.com/truce-audio/reiss-mcpherson-effects (docs: https://truce.audio/docs/examples/reiss-chorus/).

### Future DSP work (ideas, not started)

Suggested order: width → feedback → high-pass → multiple voices. Each new building block goes in `src/dsp/` with its
own tests, like the delay line and LFO.

- **Width** (stereo LFO offset, 0-180°): replaces the fixed 90° `STEREO_OFFSET`. Changing it with `set_phase`
  mid-playback makes the right LFO jump (click); use one shared phase with the offset added at read time, or smooth
  the offset.
- **LFO waveform** (sine / triangle): a triangle gives a constant pitch offset that flips at the peaks (see the LFO
  waveform decision).
- **Output / wet level** in dB: one multiply after the mix; dB → linear conversion plus smoothing. Related question:
  linear vs equal-power (`cos`/`sin`) mix law. Linear is right for correlated signals and keeps mix 0 / 1 exact;
  equal-power avoids the −3 dB dip at 50% for uncorrelated signals; a chorus is partly correlated (comb filtering).
- **Feedback** (wet back into the delay line): must stay < 1 (mind Hermite overshoot), needs NaN protection (one NaN
  would circulate forever) and denormal handling on the decaying tail; same-channel vs cross (L→R) feedback.
- **High-pass on the wet path**: first filter (one-pole or biquad) with per-channel state; removes low-end mud; can sit
  inside the feedback loop.
- **Tempo-synced rate**: note lengths (1/4, 1/8, ...) → Hz from the host's BPM (`ProcessContext` transport).
- **Multiple voices** (2-4 taps per channel with spread LFO phases): `Voice` struct / arrays of state, gain
  normalisation as voices are added, CPU cost. The lush "ensemble" sound.
- **Vintage / BBD character**: band-limiting, mild saturation, noise. Open-ended.

### Delay line (`src/dsp/delay_line.rs`), decisions made

Status: **done**. `new` / `write` / `read` / `read_frac` / `reset` implemented and tested.

- **Read convention ("option A")**: `read(0)` is the most recent write; `read(d)` is the sample written `d` writes
  ago (`y[n] = x[n - d]`). Per-sample call order is write, then read.
- **Power-of-two buffer**: `new(max_delay_samples)` allocates `(max_delay_samples + 3).next_power_of_two()` samples
  and stores `mask = len - 1`. All wrapping uses `& mask`, not `%`. The rounding is internal; callers only think in
  `max_delay_samples`.
- **Why +3**: +1 because `read(0)` occupies a slot under option A, +2 for the two older taps cubic Hermite needs
  (`d + 1`, `d + 2`) at the maximum delay.
- **Fields**: `buffer`, `write_pos`, `mask`, `max_delay_samples`. `max_delay_samples` is stored (it cannot be derived
  from `len` after rounding) and is the limit for fractional reads.
- **Index math**: `write_pos.wrapping_sub(1 + delay) & mask`. `wrapping_sub` avoids the `usize` underflow panic in
  debug builds; the power-of-two mask makes the wrapped value land on the correct slot.
- **Two different limits**: the integer `read` is bounded by capacity (`delay <= mask`, checked with
  `debug_assert!`, no release clamp). The public promise is `max_delay_samples`; `read_frac` clamps its input to
  `[1.0, max_delay_samples]` (lower bound 1.0 because Hermite also reads tap `d - 1`).
- **`read_frac`**: `debug_assert!` against NaN, clamp, split `D` into `d = floor(D)` and `frac`, read taps
  `d-1, d, d+1, d+2` via `read(usize)`, 4-point cubic Hermite in Horner form. The integer signature stays as is.
  Out-of-range input (including `inf`) clamps to the nearest bound.
- **`reset`** zero-fills the buffer and leaves `write_pos` alone (reads are relative to it). No allocation.
- **`new` allocates**, so call it only from init/reset on sample-rate change, never from `process()`.
- **Tests**: impulse after delay, correctness across wraparound (ramp), reset clears history, and `max_delay`
  readable (including `max + 1` / `max + 2` headroom, for sizes around power-of-two boundaries). `read_frac`: DC input
  stays constant, integer delays match `read`, ramp interpolates exactly across wraparound, out-of-range clamps.

### LFO (`src/dsp/lfo.rs`), decisions made

Status: **done**. `new` / `set_rate` / `set_sample_rate` / `set_phase` / `next` / `reset` implemented and tested.

- **Fields**: `phase`, `phase_inc`, `sample_rate`. The rate in Hz is not stored; `set_sample_rate` recovers it as
  `sample_rate * phase_inc` (tiny float round-trip error accepted).
- **Phase in cycles**, `[0, 1)`, not radians. Stereo offset is a fraction of a cycle (0.25 = 90°).
- **`phase_inc = rate_hz / sample_rate`**, computed once in the setter so `next()` only adds and wraps.
- **Single guard**: `set_sample_rate` stores the new rate, then goes through `set_rate`, so every write to
  `phase_inc` is bounded in one place.
- **Bounds `phase_inc` to `[0, 0.5]`** (0 Hz to Nyquist). This is a safety range protecting `next()`'s invariant, not
  the musical range; the musical range (~0.1-5 Hz) belongs on the `rate` param. Negative rates clamp to 0 (no
  reverse LFO).
- **NaN handling**: `.max(0.0).min(0.5)`, not `clamp`, because `clamp` passes NaN through while `max`/`min` return
  the non-NaN argument. `max` first, so NaN → 0.0 (LFO stops) rather than 0.5. `debug_assert!` catches NaN in tests.
  Do not let clippy's `manual_clamp` turn this back into `clamp`.
- **`set_phase`** wraps with `rem_euclid(1.0)` (`fract` would leave negatives negative). It can return exactly `1.0`
  for tiny negative inputs, so `next()` must treat phase 1.0 the same as 0.0.
- **`next()` contract**: returns the value at the current phase in `[-1, 1]`, then advances. Given
  `phase <= 1` and `phase_inc <= 0.5`, a single "if >= 1, subtract 1" wrap is enough.
- **`f32` phase accumulator** is accepted: at very low rates the per-step rounding gives ~1% rate error, inaudible
  for an LFO. Tests compare periods with a tolerance.
- **Waveform: sine** (`sin(2π · phase)`). Chosen for the classic smooth chorus: pitch deviation follows the slope of
  the delay time, and a sine's slope changes smoothly (a triangle would give a constant pitch offset that flips
  abruptly at the peaks). Phase 1.0 and 0.0 give the same value, so the `rem_euclid` edge case is harmless.
- **`reset()` sets phase to 0** and leaves `phase_inc` / `sample_rate` alone. `Lfo` knows nothing about stereo: the
  L/R offset is owned by `Chorus`, whose `reset()` must reset both LFOs and then re-apply the offset with `set_phase`
  on the right one, or the stereo width collapses after every transport stop.
- **Tests**: `set_rate` math, clamping to 0 / Nyquist / `inf`, NaN → 0 (release only, `cargo test --release`),
  sample-rate change keeps the rate in Hz and the Nyquist bound, `set_phase` wraps. `next`: value before advancing,
  phase 1.0 behaves like 0.0, output and phase stay in range, period matches the rate (0.5% tolerance), rate 0 is
  constant. `reset`: phase back to 0, rate kept.

### Chorus (`src/dsp/chorus.rs`), decisions made

Status: **done** (DSP side). `new` / `set_sample_rate` / `set_rate` / `set_delay_ms` / `set_depth_ms` / `set_mix` /
`reset` / per-sample `process(left, right) -> (left, right)` implemented and tested (debug and `--release`).
Wired into `lib.rs` (see "Plugin integration" below).

- Owns the stereo LFO phase offset (see LFO `reset()` above).
- **Bipolar modulation**: `delay = base + depth * lfo`, so `base` is the centre and the delay swings `base ± depth`.
- **Maximums as `const`s**: base 25 ms, depth 5 ms. Delay lines are sized for 25 + 5 = 30 ms at the current sample
  rate, converted with `ceil`. Param ranges and buffer sizing read the same consts so they cannot drift apart.
- **Base > depth is guaranteed by the param ranges** (option "a"), not by clamping at runtime: base min 7 ms > depth
  max 5 ms, so the shortest delay is 2 ms and the params stay independent. `read_frac`'s clamp to 1 sample is only a
  safety net.
- **Mix law: linear crossfade**, `dry * (1 - mix) + wet * mix` (exact at mix = 1, unlike `dry + mix * (wet - dry)`).
  100% is allowed (pure wet = vibrato). `Chorus` takes mix as **0-1**; `lib.rs` converts from the 0-100% param.
- **Mix = 0 is bit-exact via an early return of the inputs**, because no formula is: `-0.0 + 0.0` gives `+0.0`, and
  `0 * inf` gives NaN. The branch comes **after** the state updates: delay lines are written and both LFOs advance
  every sample even at mix 0, so raising the mix later gives no stale audio or modulation jump.
- **`process` order per sample**: `next()` on both LFOs (exactly once each), delay per channel
  `delay_samples + depth_samples * lfo`, `write` the dry input, `read_frac(delay)`, mix. No clamp on the delay
  (param ranges keep it in 2-30 ms; `read_frac` clamps as a safety net). No guard against NaN input from the host
  (without feedback a NaN leaves after one delay); revisit when adding `feedback`.
- **Params (range, default)**: `rate` 0.1-5 Hz (consider a skewed/log range), default 0.8 Hz; `depth` 0-5 ms,
  default 2 ms; `delay` 7-25 ms, default 15 ms; `mix` 0-100%, default 50%.
- **Stereo offset**: `const` 0.25 cycles (90°), right LFO ahead of left. Becomes the `width` param later (see below).
- **ms → samples in the setters**, not per sample (same pattern as `Lfo::phase_inc`). `process` only works in
  samples. Fields keep both: `delay_ms` / `depth_ms` (to recompute) and `delay_samples` / `depth_samples`.
- **Two conversions**: `ms_to_samples` returns an exact `f32` for params (no rounding, so smoothed params glide
  instead of stepping a whole sample at a time, which would zipper). `max_delay_samples(sample_rate)` rounds up
  (`ceil` → `usize`) and is used only for delay-line capacity.
- **`set_sample_rate`**: asserts the rate, stores it **first**, re-creates both delay lines, forwards the rate to
  both LFOs, then calls `set_delay_ms(self.delay_ms)` / `set_depth_ms(self.depth_ms)` so conversion and clamping
  live only in the setters.
- **`new`** builds the struct, calls `set_rate(DEFAULT_RATE)` (`Lfo::new` starts at 0 Hz), then `reset()` (which
  applies the stereo offset). Defaults are `const`s.
- **Setter guarding: same policy as `Lfo`**. Every setter clamps to its valid range NaN-safely (`max` then `min`)
  with a `debug_assert!` against NaN, so `process` never sees a bad value. NaN falls back to the lower bound:
  delay 7 ms, depth 0, mix 0 (dry). Clippy's `manual_clamp` flags this; do not let `clippy --fix` turn it into
  `clamp`.
- **Tests**: setters convert and clamp (incl. ±`inf`), NaN → lower bound (release only), `new` sizes delay lines
  for 30 ms and applies the stereo offset and default rate, `reset` clears lines and restores the offset,
  `set_sample_rate` resizes and keeps delay/depth in ms. `process`: silence in → silence out, mix 0 bit-exact
  (`to_bits`, incl. `-0.0`), mix 0 keeps state running, pure wet with depth 0 is a plain delay (44.1/48/96 kHz),
  mix 0.5 averages, stereo offset makes channels differ, depth changes the output, jumping/out-of-range params stay
  finite and within ±1.5 (Hermite overshoot), `reset` silences the tail, NaN params keep output finite (release).
- Range consts (`MIN_DELAY_MS`, `MAX_DELAY_MS`, `MAX_DEPTH_MS`) are `pub(crate)` so `lib.rs` can use them.
- Open for later: when `width` becomes a param, changing the offset with `set_phase` mid-playback makes the right LFO
  jump. Consider one shared phase with the offset added at read time, or smoothing the offset.

### Plugin integration (`src/lib.rs`), decisions made

Status: **done**. auval passes (`aufx CCho Leem`). pluginval and
clap-validator are not installed on this machine yet.

- **Params**: `rate` `log(0.1, 5)` Hz, default 0.8, `smooth = "log(20)"`; `depth` `linear(0, 5)` ms, default 2,
  `exp(50)`; `delay` `linear(7, 25)` ms, default 15, `exp(50)`; `mix` `linear(0, 1)` with unit `%` (stored 0-1,
  displayed 0-100%, so no conversion), default 0.5, **`linear(20)`**. Ranges are string literals in the attribute;
  `tests::param_ranges_match_dsp_consts` keeps them equal to the DSP consts.
- **Mix smoothing must stay linear**: truce's linear smoother lands exactly on its target, exponential only
  approaches it, so with `exp` the bit-exact mix-0 path never triggers
  (`mix_ramped_to_zero_becomes_bit_exact_passthrough` fails with `exp`; verified).
- **State**: `CrazyChorus { chorus, sample_rate }` with a manual `Default` building `Chorus` at a 48 kHz placeholder
  (truce requires `DspState: Default` and `init` has no sample rate). `reset` sets the real rate
  (`set_sample_rate`, re-allocates) and calls `Chorus::reset`.
- **`process`**: `buffer.for_each_frame_io::<2, 2, _>`; per sample, `read()` every param once, call the setters,
  then `Chorus::process`. `for_each_frame_io` repeats the last input channel for a mono input.
- **Bus layouts**: stereo -> stereo and mono -> stereo (the user chose this; no mono -> mono).
- **`tail()`**: `ceil(sample_rate * 30 ms)` samples. Latency 0 (default).
- **Editor**: built-in `GridLayout` with four knobs. `truce.toml` `vst3_subcategory = "Modulation"`.
- **Tests** (`tests/plugin.rs`, `truce-test` dev-dependency): truce static checks (info, AU codes, bus config,
  params, state round-trip / corrupt / empty state), silence, impulse at 10 ms at 44.1/48/96 kHz, output identical
  bit-for-bit across block sizes 1/64/512/4096 with mid-run automation, mix ramp to 0 becomes bit-exact,
  extreme automation finite and below 1.5, tail silent after 31 ms, reported tail, mono -> stereo equals duplicated
  stereo input and still has width, and `process_is_realtime_clean` (only with `--features rt-paranoid`).

## Testing plan

- Unit tests on the DSP struct: silence in gives silence out; mix = 0 equals input exactly; no NaN/inf with extreme params; output stays bounded.
- `truce_test::PluginDriver` tests: impulse response shape, 44.1/48/96 kHz, block sizes 1/64/512/4096, mono and stereo bus layouts, state save/restore.
- `cargo truce validate` must pass (auval is what Logic enforces) before any manual DAW test.
- Final check by hand: load the AU in Logic, automate a param, save/reload a project.

## GUI (Slint), decisions made

Status: **done (v1)**, written by the agent at the user's request. Renders headlessly via `truce_test::screenshot!`.

- **Toolkit: Slint** via `truce-slint` 6.3. `slint` is pinned to `=1.15.1` (must match truce-slint; the code
  generated from `ui/main.slint` refers to the `slint` crate). `build.rs` calls `truce_slint_build::compile`.
  Rejected: egui (tool-like look), iced (steeper), web/React (not built into truce; WKWebView inside Logic's
  out-of-process AU host is untested risk; per-instance browser cost).
- **Look**: minimal and modern, Catppuccin Mocha (https://catppuccin.com/palette) with **peach** (`#fab387`) accents;
  colours live in the `Mocha` global in `ui/main.slint`. Font: bundled JetBrains Mono. Fixed size **360 x 156**
  (`EDITOR_SIZE`, `.resizable(false)`; AU v2 editors can't be resized by the host).
- **Own knob** (`ChorusKnob`), not truce's `Knob`: 270° arc, peach value arc, dot indicator; drag vertically
  (Shift = fine), scroll to nudge, double-click resets to the param default.
- **Edit gestures**: callbacks `begin-edit` / `edit` / `end-edit` / `reset` with a knob index; Rust maps the index via
  `KNOBS` (order must match the `.slint` file) to `begin_edit` on press, `set_param` while dragging, `end_edit` on
  release, so a drag is one automation gesture. truce-slint's `bind!` was avoided because it sends begin/set/end on
  every mouse move.
- **Sync**: the per-frame closure sets normalized values and readout strings from the host
  (`get_param` / `get_param_plain`), so automation and presets move the knobs. Readouts are formatted in Rust
  (`format_rate`: "0.80 Hz" below 1 Hz, else one decimal; ms with one decimal; mix as %).
- Web (HTML/React) UI remains possible later through a custom `Editor` + `wry`; prototype one knob inside Logic first.

## Open questions (ask the user, do not guess)

- Plugin name, vendor name, and the 4-char `au_manufacturer` / `fourcc` codes.
- Is a Developer ID Application certificate available (needed for AU v3 and for distribution)?
- Which hosts besides Logic matter for VST3 (Ableton, Reaper, other)?
- Is this for personal use or distribution (affects signing, notarization, `cargo truce package`)?

## Working agreements for AI agents

- The educational rules at the top of this file take priority over everything below.
- Check the truce docs or rustdoc before explaining or writing framework code from memory.
- After the user changes DSP code, suggest running `cargo clippy` and `cargo test`; after identifier or format
  changes, suggest `cargo truce validate`. Run them yourself only to verify code you were asked to write.
- Keep the "decisions made" sections of this file up to date when the user settles a design choice.
- Ask before changing the framework choice, AU identifiers, or signing setup.
