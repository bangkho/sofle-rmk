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
- OLED: SSD1306 128x32 on I2C0, `SDA=D2`, `SCL=D3`, addr `0x3C`. On nice!nano v2
  the OLED rail is gated by **EXT_POWER = P0.13**: without it driven high the
  display stays dark. RMK has no `ext_power` keycode — the pin is a static
  output in `keyboard.toml`.
- Battery: nice!nano v2 measures through the internal **VDDH** divider
  (`battery_adc_pin = "vddh"` on each split side, using the chip's 1:5 divider).

## Keymap

- Layers: 0 base, 1 lower, 2 raise, 3 adjust. Adjust is a tri-layer
  (`[behavior.tri_layer]`), not ZMK's `conditional_layers`.
- Home row mods on the base layer: `A S D F` → GUI/Alt/Ctrl/Shift,
  `J K L ;` → Shift/Ctrl/Alt/GUI (GACS, mirrored). Tuned via
  `[behavior.morse]` — flow tap plus prior-idle is what keeps fast rolls from
  firing modifiers. Tapping feels late? lower `hold_timeout`; modifiers fire
  accidentally? raise `prior_idle_time`.

## Files

- `keyboard.toml` — single source of truth: matrix, layout, keymap, encoders,
  display, outputs, split, battery, behaviors.
- `vial.json` — Vial's view of the layout. **Must stay in sync with
  `[layout].map`.** Generate it, never hand-edit:
  `rmkit layout convert --to-vial keyboard.toml`.
- `src/central.rs`, `src/peripheral.rs` — one-liner `#[rmk_central]` /
  `#[rmk_peripheral(id = 0)]` entry points. Config lives in TOML; only touch
  these if a feature genuinely needs the Rust API.
- `memory.x`, `build.rs`, `Makefile.toml`, `.cargo/config.toml` — bootloader
  layout, Vial embedding, UF2 generation, target selection.

## Build

Requires `thumbv7em-none-eabihf`, `flip-link`, and an ARM GNU toolchain
(`arm-none-eabi-gcc`/`-ar`) for `p256-cortex-m4-sys`. If the toolchain is not on
`PATH`, it lives at `~/.local/toolchains/arm-gnu-toolchain-*/bin`.

```sh
cargo build --release          # both halves
cargo make uf2 --release       # sofle-rmk-{central,peripheral}.uf2
```

There is no host-side unit-test suite; the build *is* the check. A successful
`cargo build --release` for both bins plus a `check output` with no
`keyboard.toml` validation error is the acceptance criterion.

## Configuration gotchas

- **Docs on rmk.rs describe `main`, which is ahead of the published crate.** The
  pinned `rmk 0.9.0` pulls `rmk-config 0.8.0`, whose schema differs. Ground truth
  is `~/.cargo/registry/src/*/rmk-config-0.8.0/src/lib.rs`, not the website.
  Concretely: split batteries are flat fields on `[split.central]` /
  `[[split.peripheral]]` (`battery_adc_pin`, `battery_user_description`,
  `adc_divider_*`), *not* a `[split.central.battery]` table; and named morse
  profiles use `mode` / `hold_timeout_ms` / `gap_timeout_ms`, not
  `permissive_hold` / `hold_timeout`.
- `board = "nice!nano_v2"` pulls a preset that enables BLE and storage. Set
  `board` **or** `chip`, never both.
- For split keyboards, connectivity lives in `[split.central.*]` /
  `[[split.peripheral.*]]` tables. A top-level `[battery]`, `[output]` or
  `[display]` is rejected.
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

## Commits

Conventional commits, one logical change each. Keep `keyboard.toml` and
`vial.json` changes in the same commit. Do not push unless asked.
