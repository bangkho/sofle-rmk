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
# one-time, host-side
rustup target add thumbv7em-none-eabihf
cargo install cargo-make flip-link
# p256-cortex-m4-sys compiles C, so an ARM GNU toolchain is required:
brew install --cask gcc-arm-embedded      # needs sudo
# ...or, without sudo, unpack it into ~/.local/toolchains and put its bin/ on PATH
```

Then:

```sh
cargo build --release          # raw ELF for both halves
cargo make uf2 --release       # -> dist/sofle-rmk-<version>-{central,peripheral}-<YYYYMMDD-HHMMSS>.uf2
```

`dist/` is gitignored; the version and build time are part of the file name, so
nothing is overwritten.

## Flashing

Both halves run **different** firmware: the left half is the BLE central, the
right half is the peripheral. Never swap them.

### 1. Get a bootloader

nice!nano v2 ships the [Adafruit nRF52 bootloader](https://github.com/adafruit/Adafruit_nRF52_Bootloader),
which exposes the board as a USB mass-storage drive — no programmer, no SWD, no
`dfu-util` needed. If you ever replaced it, re-flash that bootloader first
(via SWD); RMK's `memory.x` assumes it (FLASH starts at `0x1000`).

### 2. Enter the bootloader

1. Plug the half you are flashing into the computer with a **USB data cable**
   (charge-only cables show no drive). Any 3.3 V-tolerant port works.
2. **Double-tap the RESET button** on the nice!nano — two quick presses, within
   about half a second. A reset-then-reset is what triggers the bootloader;
   a single press just reboots the keyboard.
3. A mass-storage volume named **`NICENANO`** appears (macOS: `/Volumes/NICENANO`).
   Only one half is in the bootloader at a time, so do them one after the other.

### 3. Copy the right image

```sh
# newest build, if you have several
ls -t dist/*.uf2 | head -2

cp dist/sofle-rmk-<version>-central-<stamp>.uf2    /Volumes/NICENANO/   # left half
cp dist/sofle-rmk-<version>-peripheral-<stamp>.uf2 /Volumes/NICENANO/   # right half
```

Or drag the file onto the `NICENANO` drive in Finder. The board writes the image,
reboots on its own and the drive disappears. Some systems report an I/O error as
the volume detaches mid-copy — that is expected; the image was already written.

Order does not matter. Flash one half, then repeat steps 2–3 for the other.

### 4. Verify

The board re-enumerates as a USB HID keyboard. On macOS,
`system_profiler SPUSBDataType | grep -A3 -i pondlabs` should show
**PondLabs Sofle**. If the keyboard types on the wrong columns or not at all, the
`central` image went to the right half or vice versa — reflash correctly.

### 5. First boot and pairing

Nothing to configure: the central scans for the peripheral and bonds to it
automatically (the BLE address comes from each chip's factory ID). Attach a
battery and switch the keyboard on; both halves work over BLE with no host cable.

To pair the keyboard with a computer:

1. Hold the **RAISE** thumb key (right half, second thumb key from the inner
   edge) to reach layer 2.
2. On that layer, the left half's top row is: outermost key = **clear the current
   profile's bond**, then **profile 0…4**.
3. Tap a profile key to switch to it, or hold one for ~5 s to clear its bond and
   start advertising; then pair from the host.

USB is used automatically whenever the cable is plugged in, so no key is needed
to switch output. Bonds live in the chip's internal flash and survive reflashing.

### Optional: software bootloader entry

Double-tapping RESET always works. If you would rather not reach for the button,
bind `Bootloader` on a layer — RMK's `adafruit_bl` feature (already enabled) makes
that keycode jump straight into the bootloader on release.

### Optional: flashing over SWD

`.cargo/config.toml` sets `probe-rs run --chip nRF52840_xxAA` as the runner, so
with a debug probe wired to SWDIO/SWCLK/GND you can `cargo run --release --bin
central` to flash and get defmt RTT logs. Not needed for normal use.

### Troubleshooting

| Symptom | Cause |
|---|---|
| No `NICENANO` volume | charge-only cable, or the two RESET taps were too slow. Try another cable/port; `diskutil list` to confirm. |
| Copy "fails" with I/O error | normal — the drive detaches when the write finishes |
| Nothing types after flashing | wrong image on that half (`central` ↔ `peripheral`) |
| OLED stays dark | the rail is gated by EXT_POWER (P0.13); it lights once RMK is running on that half |
| Half missing from typing | peripheral not bonded or its battery is flat — reflash the peripheral, then power-cycle the central |
| Want a factory reset | hold a profile key for ~5 s to clear its bond, or `ClearEeprom` if you bind it |

## Layout

`keyboard.toml` is the single source of truth for the matrix, layout, keymap and
all peripherals. `vial.json` describes the same layout for the [Vial](https://get.vial.today/)
keymap editor — keep the two in sync when you change the layout.

## Licence

MIT — see [LICENSE](LICENSE).

## Credits

- [RMK](https://github.com/rmk-rs/rmk) by Haobo Gu
- [Sofle Keyboard](https://github.com/josefadamcik/SofleKeyboard) by Josef Adamčík
