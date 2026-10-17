# sofle-rmk

[RMK](https://rmk.rs) (Rust Mechanical Keyboard) firmware for a **Sofle V2** split
keyboard running **nice!nano v2** (nRF52840, "Pro Micro nRF52840") controllers.

Wireless split: the two halves talk to each other over BLE, the central half talks
to the host over BLE or USB (USB takes over when the cable is plugged in).
The peripheral half needs no USB connection to the host — it only needs power.

Feature parity target with the ZMK config in `../sofle-config-zmk`:

- Sofle V2 matrix, 60 keys, BLE split
- 4 layers (base / lower / raise / adjust tri-layer)
- Left and right rotary encoders
- SSD1306 128x32 OLED on both halves
- Battery reporting on both halves (nice!nano v2 VDDH)
- Vial keymap editing
- Home row mods

## Hardware

| | |
|---|---|
| MCU | nice!nano v2 (nRF52840) on both halves |
| Central | left half |
| Peripheral | right half |
| Split link | BLE (RMK does not support wired serial split on nRF52) |
| Row pins | D5, D6, D7, D8, D9 |
| Left col pins | D19, D18, D15, D14, D16, D10 |
| Right col pins | D10, D16, D14, D15, D18, D19 |
| OLED | SSD1306 128x32 on I2C (D2 = SDA, D3 = SCL), powered by EXT_POWER (P0.13) |
| Left encoder | A = D21, B = D20 |
| Right encoder | A = D20, B = D21 |
| Battery | nice!nano v2 VDDH |

## Build

```sh
# one-time
rustup target add thumbv7em-none-eabihf
cargo install cargo-make flip-link
brew install --cask gcc-arm-embedded   # needed by p256-cortex-m4-sys

# build both halves
cargo build --release
cargo make uf2 --release               # -> sofle-rmk-{central,peripheral}.uf2
```

## Flash

1. Put the board into the bootloader: double-tap reset.
2. Drag `sofle-rmk-central.uf2` onto the `NICENANO` drive of the **left** half.
3. Drag `sofle-rmk-peripheral.uf2` onto the **right** half.

Flash order does not matter; the two halves bond on first boot.

## Layout

`keyboard.toml` is the single source of truth for the matrix, layout, keymap and
all peripherals. `vial.json` describes the same layout for the [Vial](https://get.vial.today/)
keymap editor — keep the two in sync when you change the layout.

## Credits

- [RMK](https://github.com/rmk-rs/rmk) by Haobo Gu
- [Sofle Keyboard](https://github.com/josefadamcik/SofleKeyboard) by Josef Adamčík
