# AGENTS.md — sofle-rmk

RMK (Rust) firmware for a **Sofle V2** split keyboard with **nice!nano v2**
(nRF52840) controllers. Wireless split only.

## Hardware facts (do not re-derive)

- Central = **left** half, peripheral = **right** half. Swap by moving the
  `row_offset`/`col_offset` blocks, nothing else.
- Split link is **BLE**. RMK's serial/wired split module is compiled only when
  the `_ble` feature is off, and every nRF52840 RMK feature enables `_ble`, so a
  cable split is impossible on this controller. Do not propose one.
- Pin names are Pro Micro `D<n>`; RMK wants `P<port>_<pin>`. Mapping for
  nice!nano v2 (ZMK `arduino_pro_micro_pins.dtsi`):

  | D | nRF | D | nRF | D | nRF |
  |---|---|---|---|---|---|
  | D0 | P0.08 | D6 | P1.00 | D14 | P1.11 |
  | D1 | P0.06 | D7 | P0.11 | D15 | P1.13 |
  | D2 | P0.17 | D8 | P1.04 | D16 | P0.10 |
  | D3 | P0.20 | D9 | P1.06 | D18 | P1.15 |
  | D4 | P0.22 | D10 | P0.09 | D19 | P0.02 |
  | D5 | P0.24 | | | D20 | P0.29 |
  | | | | | D21 | P0.31 |

- Matrix: rows `D5,D6,D7,D8,D9`; left cols `D19,D18,D15,D14,D16,D10`; right
  cols `D10,D16,D14,D15,D18,D19`. Diode direction col2row (RMK default).
- Encoders: left `A=D21, B=D20`; right `A=D20, B=D21`. Encoder index order is
  central first, then each peripheral — so left = 0, right = 1.
- **Display: central only.** One SSD1306 128x32 on I2C0 (`SDA=D2`/`P0_17`,
  `SCL=D3`/`P0_20`, addr `0x3C`, `TWISPI0`), rail gated by **EXT_POWER = P0.13**
  (static output in `keyboard.toml`; RMK has no `ext_power` keycode). It is
  driven by `src/render.rs::SplitStatusRenderer` — two battery rows (`bat`, and
  `peer` relayed over the split link) plus `SPLIT OK/LOST, H<n>|ADV|USB`.
  **Never** add `[split.peripheral.display]` / `[[split.peripheral.output]]`: the
  right half has no panel, and a half that initialises I2C with nothing answering
  on the bus hangs outright (radio with it, so keys stick and the link drops).
  That is also why `battery_adc_pin = "vddh"` alone is safe there.
- No RGB: the Sofle V2 PCB has no LEDs and RMK's RGB keycodes are no-ops.
- Battery: nice!nano v2 measures through the internal **VDDH** divider
  (`battery_adc_pin = "vddh"` on each split side, using the chip's 1:5 divider).

## Keymap

- Layers: 0 base, 1 lower, 2 raise, 3 adjust. Adjust is a tri-layer
  (`[behavior.tri_layer]`), not ZMK's `conditional_layers`.
- **No home row mods.** `A S D F` / `J K L ;` are plain keys; `G` and `H` carry
  no modifier either. Mods live on the bottom rows and thumbs (`LGui/LCtrl/LAlt/
  LShift` left, mirrored right). `[behavior.morse]` is gone with them — re-adding
  `MT()` keys means re-adding the block; on this firmware flow tap plus
  prior-idle is what kept fast rolls from firing mods, and `permissive_hold`
  resolved rolls as holds.

## Files

- `keyboard.toml` — single source of truth: matrix, layout, keymap, encoders,
  display, outputs, split, battery, behaviors.
- `vial.json` — Vial's view of the layout. **Must stay in sync with
  `[layout].map`.** Generate it, never hand-edit:
  `rmkit layout convert --to-vial keyboard.toml`.
- `src/central.rs`, `src/peripheral.rs` — one-liner `#[rmk_central]` /
  `#[rmk_peripheral(id = 0)]` entry points. Config lives in TOML; only touch
  these if a feature genuinely needs the Rust API. `central.rs` additionally
  declares `mod render;`.
- `src/render.rs` — `SplitStatusRenderer`, the **central's** OLED screen, picked
  up via `renderer = "crate::render::SplitStatusRenderer"` (a path with `::` is
  used verbatim, a bare name is shorthand for `::rmk::display::<name>`). RMK's
  built-in `OledRenderer` folds the split and host links into one indicator, so
  on the central it shows the host link and hides whether the peripheral half is
  up; this one prints the central's battery (`bat`), the peripheral's battery
  (relayed over the split link as `peer`, needs `split` + `_ble`) and
  `SPLIT OK/LOST, H<n>|ADV|USB`.
- `memory.x`, `build.rs`, `Makefile.toml`, `.cargo/config.toml` — bootloader
  layout, Vial embedding, UF2 generation, target selection.

## Build

Requires `thumbv7em-none-eabihf`, `flip-link`, and an ARM GNU toolchain
(`arm-none-eabi-gcc`/`-ar`) for `p256-cortex-m4-sys`. If the toolchain is not on
`PATH`, it lives at `~/.local/toolchains/arm-gnu-toolchain-*/bin`.

```sh
cargo build --release          # raw ELF for both halves
cargo make uf2 --release       # dist/sofle-rmk-<version>-{central,peripheral}-<YYYYMMDD-HHMMSS>.uf2
```

`dist/` is gitignored. The crate version and the build time are part of the file
name, so successive builds never overwrite each other. Never commit firmware
images.

Releases are tag-driven: `.github/workflows/release.yml` builds on a `v*` tag and
attaches the UF2s to a GitHub release. It fails if the tag does not equal
`v<crate version>`, so bump `version` in `Cargo.toml` and tag to match.

There is no host-side unit-test suite; the build *is* the check. A successful
`cargo build --release` for both bins plus a `cargo make uf2 --release` that
writes both images is the acceptance criterion.

### Debug log over USB

A keyboard with no probe is otherwise unobservable, so log to USB CDC when
hunting a firmware fault. `defmt` and `log` are mutually exclusive in `rmk`, and
`rmk-macro` reads the `rmk` dependency's features out of **this manifest**, so
passing `--features rmk/usb_log` on the command line does nothing. Instead build
with `default-features = false` and an explicit feature list — `storage`,
`vial`, `host_lock`, `watchdog`, `log`, `usb_log` — then:

```sh
stty -f /dev/cu.usbmodemXXXX 115200 raw -echo && cat /dev/cu.usbmodemXXXX
```

Only `dfu`/`usb_log` bring USB up on a peripheral, so a release peripheral never
enumerates on USB at all; one USB keyboard device for two halves is normal.
`log` traffic is capped at a 1024-byte pipe and dropped beyond that, so a silent
port only means "no new log lines", never "hung".

## Configuration gotchas

- **Docs on rmk.rs describe `main`, which is ahead of the published crate.** The
  pinned `rmk 0.9.0` pulls `rmk-config 0.8.0`, whose schema differs. Ground truth
  is `~/.cargo/registry/src/*/rmk-config-0.8.0/src/lib.rs`, not the website.
  Concretely: split batteries are flat fields on `[split.central]` /
  `[[split.peripheral]]` (`battery_adc_pin`, `battery_user_description`,
  `adc_divider_*`), *not* a `[split.central.battery]` table; and `MorseProfile`
  is spelled `enable_flow_tap` / `prior_idle_time` / `unilateral_tap` /
  `permissive_hold` / `hold_on_other_press` / `normal_mode` / `hold_timeout` /
  `gap_timeout` (all durations as `"200ms"` strings) — there is no `mode`,
  `hold_timeout_ms` or `gap_timeout_ms`. The defaults sit directly under
  `[behavior.morse]`; per-name overrides go in `[behavior.morse.profiles.<name>]`.
  Verified against `rmk-config 0.8.0`, whose `deny_unknown_fields` fails the
  build on any other spelling.
- `board = "nice!nano_v2"` pulls a preset that enables BLE and storage. Set
  `board` **or** `chip`, never both.
- For split keyboards, connectivity lives in `[split.central.*]` /
  `[[split.peripheral.*]]` tables. A top-level `[battery]`, `[output]` or
  `[display]` is rejected. This board uses `[display]` + `[output]` on the
  central only, and `[battery]` nowhere (batteries are flat fields per side).
- Encoder action lists must be present for **every** encoder on a layer or
  omitted entirely; a partial list fails validation.
- Every `[layout].map` position needs exactly one key on every layer — a miscount
  is caught at macro expansion, not by TOML parsing, so the error surfaces as a
  `#[rmk_central]` attribute panic.
- RMK has no RGB/backlight keycodes (they parse but do nothing) and the Sofle V2
  PCB has no LEDs. Layer 3 in the ZMK config used `&rgb_ug` / `&ext_power`; those
  map to `No` here.
- `User0..N` keycodes drive BLE profiles: `N` profiles → `User0..N-1` select,
  `UserN` next, `UserN+1` previous, `UserN+2` clear bond, `UserN+3` USB/BLE
  toggle. `ble_profiles_num = 5` matches the ZMK config's `BT_SEL 0..4`.
- A host that paired with the ZMK firmware keeps that bond. RMK then logs
  `Long term key request reply failed, no long term key` and the host drops the
  link and retries forever — which also starves the split link, so the whole
  keyboard looks dead on Bluetooth. Forget the device in the host's Bluetooth
  settings and pair again; RMK stores its own LTK and reconnects normally after.

## Commits

Conventional commits, one logical change each. Keep `keyboard.toml` and
`vial.json` changes in the same commit. Do not push unless asked.
